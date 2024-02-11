// Welcome to
// __________         __    __  .__                               __
// \______   \_____ _/  |__/  |_|  |   ____   ______ ____ _____  |  | __ ____
//  |    |  _/\__  \\   __\   __\  | _/ __ \ /  ___//    \\__  \ |  |/ // __ \
//  |    |   \ / __ \|  |  |  | |  |_\  ___/ \___ \|   |  \/ __ \|    <\  ___/
//  |________/(______/__|  |__| |____/\_____>______>___|__(______/__|__\\_____>
//
// This file can be a nice home for your Battlesnake logic and helper functions.
//
// To get you started we've included code to prevent your Battlesnake from moving backwards.
// For more info see docs.battlesnake.com

use log::info;
use serde_json::{json, Value};
use crate::{Battlesnake, Board, Game};
use crate::board::{Direction, GameBoard};
use crate::search::sort_moves;

pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "JeffLegendPower", // TODO: Your Battlesnake Username
        "color": "#888888", // TODO: Choose color
        "head": "default", // TODO: Choose head
        "tail": "default", // TODO: Choose tail
    });
}

// start is called when your Battlesnake begins a game
pub fn start(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME START");
}

// end is called when your Battlesnake finishes a game
pub fn end(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME OVER");
}

// move is called on every turn and returns your next move
// Valid moves are "up", "down", "left", or "right"
// See https://docs.battlesnake.com/api/example-move for available data
pub fn get_move(_game: &Game, turn: &i32, _board: &Board, you: &Battlesnake) -> Value {

    let mut game_board: GameBoard = GameBoard::new(_board.width, _board.height as i32, _board.food.clone(), _board.snakes.clone(), _board.hazards.clone());
    // let possible_moves = game_board.generate_possible_moves(you);
    // // If there are no possible moves, we are dead anyway
    // if possible_moves.is_empty() {
    //     return json!({ "move": "up" });
    // }

    let sorted_moves = sort_moves(game_board, you.clone());

    // let chosen = possible_moves.choose(&mut rand::thread_rng()).unwrap();
    let chosen = &sorted_moves[0];

    // game_board.move_snake(game_board.get_snake(&you.id).clone(), chosen.clone());

    let str_chosen = match chosen {
        Direction::Up => "up",
        Direction::Down => "down",
        Direction::Left => "left",
        Direction::Right => "right",
    };

    // TODO: Step 4 - Move towards food instead of random, to regain health and survive longer
    // let food = &board.food;

    info!("MOVE {}: {}", turn, str_chosen);
    return json!({ "move": str_chosen });
}
