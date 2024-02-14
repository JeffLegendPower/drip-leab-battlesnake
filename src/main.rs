#[macro_use]
extern crate rocket;

use crate::transposition_table::TTEntry;
use log::info;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::serde::json::json;
use rocket::serde::{json::Json, Deserialize};
use rocket::State;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::sync::{Arc, Mutex};

mod board;
mod eval;
mod logic;
mod search;
mod transposition_table;

// API and Response Objects
// See https://docs.battlesnake.com/api

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Game {
    id: String,
    ruleset: HashMap<String, Value>,
    timeout: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Board {
    height: i32,
    width: i32,
    food: Vec<Coord>,
    snakes: Vec<Battlesnake>,
    hazards: Vec<Coord>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Battlesnake {
    id: String,
    name: String,
    health: i32,
    body: Vec<Coord>,
    head: Coord,
    length: i32,
    latency: String,
    shout: Option<String>,
}

struct GameState {
    game: Game,
    turn: i32,
    board: Board,
    you: Battlesnake,
    tt: Vec<TTEntry>,
    health_zobrist_table: Vec<u64>,
    zobrist_table: Vec<u64>,
}

struct SharedState {
    shared_state: Arc<Mutex<BTreeMap<String, GameState>>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord {
    x: i32,
    y: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JsonGameState {
    game: Game,
    turn: i32,
    board: Board,
    you: Battlesnake,
}

#[get("/")]
fn handle_index() -> Json<Value> {
    Json(logic::info())
}

#[post("/start", format = "json", data = "<start_req>")]
fn handle_start(start_req: Json<JsonGameState>, state: &State<SharedState>) -> Status {
    let start_time = std::time::Instant::now();
    let mut map = state.shared_state.lock().unwrap();
    map.insert(
        start_req.game.id.clone(),
        GameState {
            game: start_req.game.clone(),
            turn: start_req.turn.clone(),
            board: start_req.board.clone(),
            you: start_req.you.clone(),
            // tt: [TTEntry { zobrist: 0, best_move: board::Direction::None, friendly_health: 0, enemy_health: 0 }; 0x2000],
            tt: vec![
                TTEntry {
                    zobrist: 0,
                    best_move: board::Direction::None,
                    friendly_health: -1,
                    enemy_health: -1,
                    depth: -1,
                    score: -1,
                    flag: -1,
                    snake_head_x: -1,
                    snake_head_y: -1,
                    enemy_head_x: -1,
                    enemy_head_y: -1,
                };
                0x20000
            ],
            health_zobrist_table: vec![0; 100],
            zobrist_table: vec![0; 100],
        },
    );
    logic::start(map.get_mut(&start_req.game.id).unwrap());
    println!(
        "Started game {} in {}",
        start_req.game.id,
        start_time.elapsed().as_millis()
    );

    Status::Ok
}

#[post("/move", format = "json", data = "<move_req>")]
fn handle_move(move_req: Json<JsonGameState>, state: &State<SharedState>) -> Json<Value> {
    let mut map = state.shared_state.lock().unwrap();

    if !map.contains_key(&move_req.game.id) {
        println!(
            "Game {} not found, the /start endpoint might be backed up",
            move_req.game.id
        );
        println!("Playing default move (UP) for now...");
        return Json(json!({ "move": "up" }));
    }

    let mut_entry = map.get_mut(&move_req.game.id).unwrap();

    mut_entry.game = move_req.game.clone();
    mut_entry.board = move_req.board.clone();
    mut_entry.turn = move_req.turn.clone();

    let response = logic::get_move(mut_entry);

    Json(response)
}

#[post("/end", format = "json", data = "<end_req>")]
fn handle_end(end_req: Json<JsonGameState>, state: &State<SharedState>) -> Status {
    let mut map = state.shared_state.lock().unwrap();
    map.remove(&end_req.game.id);
    println!("Ended game {}", end_req.game.id);
    logic::end(&end_req.game, &end_req.turn, &end_req.board, &end_req.you);

    Status::Ok
}

#[launch]
fn rocket() -> _ {
    // Lots of web hosting services expect you to bind to the port specified by the `PORT`
    // environment variable. However, Rocket looks at the `ROCKET_PORT` environment variable.
    // If we find a value for `PORT`, we set `ROCKET_PORT` to that value.
    if let Ok(port) = env::var("PORT") {
        env::set_var("ROCKET_PORT", &port);
    }

    let shared_state = Arc::new(Mutex::new(BTreeMap::<String, GameState>::new()));

    // TODO this is debug
    // if env::var("RUST_LOG").is_err() {
    //     env::set_var("RUST_LOG", "info");
    // }

    env_logger::init();

    info!("Starting Battlesnake Server...");

    rocket::build()
        .manage(SharedState { shared_state })
        .attach(AdHoc::on_response("Server ID Middleware", |_, res| {
            Box::pin(async move {
                res.set_raw_header("Server", "battlesnake/github/starter-snake-rust");
            })
        }))
        .mount(
            "/",
            routes![handle_index, handle_start, handle_move, handle_end],
        )
}
