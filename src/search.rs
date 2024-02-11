use std::fs::OpenOptions;
use std::time::{Duration, Instant};
use crate::Battlesnake;
use crate::board::{Direction, GameBoard};
use crate::eval::eval;

pub fn sort_moves(mut board: GameBoard, snake: Battlesnake) -> Vec<Direction> {

    let nearest_enemy = board.snakes.iter().filter(|s| s.borrow().id != snake.id).min_by_key(|s| {
        let head = &s.borrow().head;
        let snake_head = &snake.head;
        return (head.x - snake_head.x).abs() + (head.y - snake_head.y).abs();
    });

    let mut ref_snake = board.get_snake(&snake.id);

    if nearest_enemy.is_none() {
        let possible_moves = board.generate_possible_moves(ref_snake.clone());
        // if possible_moves.len() > 0 {
        //     board.move_snake(ref_snake.clone(), possible_moves[0].clone());
        // }
        return possible_moves
    }

    let mut nodes_searched = 0;
    let mut best_move = Direction::Up;

    // let mut EVAL_TOTAL_TIME = Duration::from_secs(0);
    // let mut EVAL_TOTAL_RUNS = 0;
    // let mut MOVES_TOTAL_TIME = Duration::from_secs(0);
    // let mut MOVES_TOTAL_RUNS = 0;

    let start_time = Instant::now();

    let enemy_borrow = nearest_enemy.unwrap().borrow();
    let enemy_id = enemy_borrow.id.clone();
    drop(enemy_borrow);

    let mut depth = 1;
    // let mut best_moves: Vec<Direction> = Vec::new();
    // let mut searched_nodes: Vec<i32> = Vec::new();
    let mut score = 0;
    while start_time.elapsed().as_millis() < 250 && depth <= 50 {
        nodes_searched = 0;
        let mut temp_best_move = Direction::Up;
        // best_moves = vec![Direction::Up; depth.clone() as usize];
        // searched_nodes = vec![0; depth.clone() as usize];
        score = minimax(&mut board, &snake.id, &enemy_id, depth.clone(), 0, -9999999, 9999999,
                        true, // Doesn't matter for root node
                            &mut nodes_searched, &mut temp_best_move, start_time,
                        // &mut best_moves, &mut searched_nodes,
                            // &mut EVAL_TOTAL_TIME, &mut EVAL_TOTAL_RUNS, &mut MOVES_TOTAL_TIME, &mut MOVES_TOTAL_RUNS
        );
        depth += 1;

        // If the search cuts early, it will be bad so we will prevent that from affecting the best move
        if start_time.elapsed().as_millis() <= 290 {
            best_move = temp_best_move.clone();
        }
    }
    println!("Nodes Searched: {}, Depth {}, Best Score: {}, Best Direction: {:?}", nodes_searched, depth, score, best_move);
    // println!("Best Path: {:?}", best_moves);
    // println!("Searched Nodes: {:?}", searched_nodes);

    // print_average_timings(EVAL_TOTAL_TIME, EVAL_TOTAL_RUNS, MOVES_TOTAL_TIME, MOVES_TOTAL_RUNS);
    // println!("Minimax Time: {:?}", start_time.elapsed());

    // board.move_snake(&snake.id, moves[0].clone());

    // println!("Sorted Move: {:?}, Best Score: {}", moves[0], eval(board.clone(), &snake.id));
    // println!();

    // board.undo_move();
    // moves
    // single vector for best_move
    return vec![best_move];
}

pub fn minimax(mut board: &mut GameBoard, snake_id: &str, enemy_id: &str, mut depth: i32, ply: i32, mut alpha: i32, mut beta: i32, shouldNMP: bool,
               nodes_searched: &mut i32, best_move: &mut Direction, start_time: Instant,
               // bestPath: &mut Vec<Direction>, searchedNodes: &mut Vec<i32>,
               /*EVAL_TOTAL_TIME: &mut Duration, EVAL_TOTAL_RUNS: &mut i32, MOVES_TOTAL_TIME: &mut Duration, MOVES_TOTAL_RUNS: &mut i32*/) -> i32 {
    *nodes_searched += 1;

    let mut snake = board.get_snake(snake_id).clone();
    let mut enemy = board.get_snake(enemy_id).clone();

    if depth <= 0 || start_time.elapsed().as_millis() >= 300 {
        return if ply % 2 == 0 {
            eval(board, snake.clone(), enemy.clone())
        } else {
            -eval(board, enemy.clone(), snake.clone())
        };
    }

    // let binding = board.clone();
    // let snake = binding.get_snake(snake_id);

    let mut best_score = -999999;

    // let start_time = Instant::now();
    // println!("Ply: {}", ply);
    // let mut file = OpenOptions::new()
    //     .write(true)
    //     .append(true)
    //     .create(true)
    //     .open("/Users/ishaangoyal/RustroverProjects/starter-snake-rust/src/output.txt").unwrap();
    // if ply <= 0 {
    //     writeln!(file, "Ply: {}", ply).unwrap();
    // }
    let possible_moves = board.generate_possible_moves(snake.clone());
    // update_timing(start_time, MOVES_TOTAL_TIME, MOVES_TOTAL_RUNS);

    // Dead
    if possible_moves.is_empty() || snake.borrow().health <= 0 {
        return -10000;
    }
    // Potential game-ending branch
    if possible_moves.len() == 1 {
        depth += 1;
    }

    // // if it's a lot worse than alpha, reduce it
    // if score < alpha - 15 {
    //     depth -= 1;
    //     // beta = alpha + 1;
    // }

    // // Null Move Pruning
    if depth > 5 && shouldNMP {
        // Give the enemy snake an extra move, if we are still doing better, then this is a great position

        let nmp = -minimax(board, enemy_id, snake_id, 3, ply + 1, -beta, -alpha,
                           false,
                          nodes_searched, best_move, start_time,
                          /*EVAL_TOTAL_TIME, EVAL_TOTAL_RUNS, MOVES_TOTAL_TIME, MOVES_TOTAL_RUNS*/);

        // NMP fail-high
        if nmp >= beta {
            // println!("NMP Fail High, Score: {}", nmp);
            return beta;
        }
    }


    for dir in &possible_moves {

        // // Null-window the last 2 moves
        // if i > 1 {
        //     beta = alpha + 1;
        // }
        //
        // // Reduce the last move
        // if i > 2 {
        //     depth -= 1;
        // }

        // let id = snake.borrow().id.clone();
        // TODO theres a huge bug with movegen and i cant find out why so for now just ignore illegals
        let passed = board.move_snake(snake.clone(), dir.clone());
        if !passed {
            println!("Error in movegen, ignoring this move...");
            println!("a: {:?}, b: {:?}, c: {:?}", possible_moves.clone(), snake.borrow().head, dir.clone());
            continue
        }
        // if ply < searchedNodes.len() as i32 {
        //     searchedNodes[ply as usize] += 1;
        // }

        let new_score = -minimax(board, enemy_id, snake_id, depth - 1, ply + 1, -beta, -alpha,
                                 shouldNMP,
                                 nodes_searched, best_move, start_time,
                                 // bestPath, searchedNodes,
                                 /*EVAL_TOTAL_TIME, EVAL_TOTAL_RUNS, MOVES_TOTAL_TIME, MOVES_TOTAL_RUNS*/);
        board.undo_move();

        snake = board.get_snake(&snake_id).clone();
        enemy = board.get_snake(&enemy_id).clone();

        // TODO in the future instead of deep copying boards and having to do all this just undo actions
        // snake = board.get_snake(&snake.borrow().id).clone();
        // enemy = board.get_snake(&enemy.borrow().id).clone();

        if new_score > best_score {
            best_score = new_score;
            if ply == 0 {
                *best_move = dir.clone();
            }
            // if ply < bestPath.len() as i32 {
            //     bestPath[ply as usize] = dir.clone();
            // }
        }
        if new_score > alpha {
            alpha = new_score;
            if alpha >= beta {
                return beta;
            }
        }
    }

    return best_score;
}

// static mut EVAL_TOTAL_TIME: Duration = Duration::from_secs(0);
// static mut EVAL_TOTAL_RUNS: u64 = 0;
//
// static mut MOVES_TOTAL_TIME: Duration = Duration::from_secs(0);
// static mut MOVES_TOTAL_RUNS: u64 = 0;

// Helper function to update timing information
fn update_timing(start_time: Instant, total_time: &mut Duration, total_runs: &mut i32) {
    let elapsed_time = start_time.elapsed();
    *total_time += elapsed_time;
    *total_runs += 1;
}

// Function to print average timings
pub fn print_average_timings(EVAL_TOTAL_TIME: Duration, EVAL_TOTAL_RUNS: i32, MOVES_TOTAL_TIME: Duration, MOVES_TOTAL_RUNS: i32) {
    unsafe {
        let eval_average = EVAL_TOTAL_TIME.div_f32(EVAL_TOTAL_RUNS as f32);
        let moves_average = MOVES_TOTAL_TIME.div_f32(MOVES_TOTAL_RUNS as f32);

        println!("Eval Average Time: {:?}", eval_average);
        println!("Generate Moves Average Time: {:?}", moves_average);
    }
}