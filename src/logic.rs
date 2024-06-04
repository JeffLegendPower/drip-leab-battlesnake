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
use rand::Rng;
use serde_json::{json, Value};
use crate::{Battlesnake, Board, Game, GameState};
use crate::board::{Direction, GameBoard};
use crate::game_recorder::GameRecorder;
use crate::search::think;

pub fn info() -> Value {
    info!("INFO");

    return json!({
        "apiversion": "1",
        "author": "JeffLegendPower",
        "color": "#888888", // TODO: Choose color
        "head": "default", // TODO: Choose head
        "tail": "default", // TODO: Choose tail
    });
}

// start is called when your Battlesnake begins a game
pub fn start(game: &mut GameState) {

    // Populate the zobrist table
    let mut rng = rand::thread_rng();
    for _ in 0..(game.board.width as i32 * game.board.height as i32 * 2) {
        game.zobrist_table.push(rng.gen());
    }
    for _ in 0..100 {
        game.health_zobrist_table.push(rng.gen());
    }

    info!("GAME START");
}

// end is called when your Battlesnake finishes a game
pub fn end(_game: &Game, _turn: &i32, _board: &Board, _you: &Battlesnake) {
    info!("GAME OVER");
}

// move is called on every turn and returns your next move
// Valid moves are "up", "down", "left", or "right"
// See https://docs.battlesnake.com/api/example-move for available data
// pub fn get_move(_game: &Game, turn: &i32, _board: &Board, you: &Battlesnake) -> Value {
pub fn get_move(game: &mut GameState) -> Value {
    let board = &game.board;

    let game_board: GameBoard = GameBoard::new(board.width, board.height, board.food.clone(), board.snakes.clone(), board.hazards.clone(),
                                                   &game.zobrist_table, &game.health_zobrist_table);
    let best_move = think(&mut game.game_recorder, game_board, game.you.clone(), &mut game.tt, &mut game.killers);

    let best_move_str = match best_move {
        Direction::Up => "up",
        Direction::Down => "down",
        Direction::Left => "left",
        Direction::Right => "right",
        Direction::None => "up",
    };

    info!("MOVE {}: {}", game.turn, best_move_str);
    return json!({ "move": best_move_str });
}
