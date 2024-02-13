use std::fs::OpenOptions;
use std::time::Instant;
use std::io::Write;
use crate::Battlesnake;
use crate::board::{CellContent, Direction, GameBoard};
use crate::eval::eval;
use crate::transposition_table::TTEntry;

pub fn think(mut board: GameBoard, snake: Battlesnake, transposition_table: &mut Vec<TTEntry>) -> Direction {

    let nearest_enemy = board.snakes.iter().filter(|s| s.borrow().id != snake.id).min_by_key(|s| {
        let head = &s.borrow().head;
        let snake_head = &snake.head;
        return (head.x - snake_head.x).abs() + (head.y - snake_head.y).abs();
    });

    let ref_snake = board.get_snake(&snake.id);

    if nearest_enemy.is_none() {
        let possible_moves = board.generate_possible_moves(ref_snake.clone());
        if possible_moves.len() > 0 {
            return possible_moves[0]
        }
        return Direction::None;
    }

    let mut nodes_searched = 0;
    let mut best_move = Direction::None;

    let start_time = Instant::now();

    let enemy_borrow = nearest_enemy.unwrap().borrow();
    let enemy_id = enemy_borrow.id.clone();
    drop(enemy_borrow);

    let mut depth = 0;
    let mut score = 0;

    while start_time.elapsed().as_millis() < /*250*/60 && depth <= 50 {
        // while depth == 10 {
        depth += 1;

        nodes_searched = 0;
        let mut temp_best_move = Direction::None;
        score = minimax(&mut board, &snake.id, &enemy_id, depth.clone(), 0, -9999999, 9999999,
                        true, Direction::None, Direction::None,
                        transposition_table,
                        &mut nodes_searched, &mut temp_best_move, start_time,
        );

        // If the search cuts early, it will be bad so we will prevent that from affecting the best move
        if start_time.elapsed().as_millis() <= /*290*/70 && temp_best_move != Direction::None {
            best_move = temp_best_move.clone();
        }
    }

    println!("Nodes Searched: {}, Depth {}, Best Score: {}, Best Direction: {:?}", nodes_searched, depth, score, best_move);
    // write_eval_data(score, board.clone(), &snake.id, &enemy_id);


    return best_move;
}

pub fn minimax(board: &mut GameBoard, snake_id: &str, enemy_id: &str, mut depth: i32, ply: i32, mut alpha: i32, mut beta: i32,
               should_nmp: bool, last_move: Direction, last_2nd_move: Direction,
               transposition_table: &mut Vec<TTEntry>,
               nodes_searched: &mut i32, best_move: &mut Direction, start_time: Instant,) -> i32 {
    *nodes_searched += 1;

    let snake = board.get_snake(snake_id).clone();
    let enemy = board.get_snake(enemy_id).clone();

    if depth <= 0 || start_time.elapsed().as_millis() >= /*300*/80 {
        return if ply % 2 == 0 {
            eval(board, snake.clone(), enemy.clone())
        } else {
            -eval(board, enemy.clone(), snake.clone())
        };
    }

    let mut best_score = -999999;

    let possible_moves = board.generate_possible_moves(snake.clone());
    // update_timing(start_time, MOVES_TOTAL_TIME, MOVES_TOTAL_RUNS);

    // They moved into us so this is just punishing that to prevent it
    if (snake.borrow().head.x - enemy.borrow().head.x).abs() + (snake.borrow().head.y - enemy.borrow().head.y).abs() == 1 {
        return 10000;
    }

    // Dead
    if possible_moves.is_empty() || snake.borrow().health <= 0 {
        return -10000 + ply;
    }
    // Potential game-ending branch
    if possible_moves.len() == 1 {
        depth += 1;
    }

    let snake_head = snake.borrow().head.clone();
    let enemy_head = enemy.borrow().head.clone();

    let entry = &transposition_table[(board.zobrist_hash & 0x1FFFF) as usize];
    let tt_hit = entry.zobrist == board.zobrist_hash
        && entry.friendly_health == snake.borrow().health
        && entry.enemy_health == enemy.borrow().health
        && entry.snake_head_x == snake_head.x
        && entry.snake_head_y == snake_head.y
        && entry.enemy_head_x == enemy_head.x
        && entry.enemy_head_y == enemy_head.y;

    if ply > 0 && tt_hit && entry.depth >= depth {
        match entry.flag {
            1 => {
                alpha = alpha.max(entry.score);
            }
            2 => {
                beta = beta.min(entry.score);
            }
            3 => {
                return entry.score;
            }
            _ => {}
        }
        if alpha >= beta {
            return entry.score;
        }
        // if entry.score >= beta {
        //     return beta;
        // }
        // if entry.score <= alpha {
        //     return alpha;
        // }
        // return entry.score;
    }

    let mut tt_flag = 1;

    let mut scored_moves: Vec<(&Direction, i32)> = possible_moves.iter().map(|dir| (dir,
                                                                                    // NO TONKAs, no AUs, no NEWTONMETERS, no INVERSEKILOJOULESPERMETERSSQUARED, no GOLDMAN
                                                                                    if tt_hit && dir == &entry.best_move {
                                                                                        1_000_000
                                                                                        // } else if *dir == last_2nd_move { // Tempo bonus
                                                                                        //     100_000
                                                                                    } else {
                                                                                        0
                                                                                    }
    )).collect();

    // // Null Move Pruning
    if depth > 5 && should_nmp {
        // Give the enemy snake an extra move, if we are still doing better, then this is a great position

        let nmp = -minimax(board, enemy_id, snake_id, 3, ply + 1, -beta, -alpha,
                           false, last_2nd_move.clone(), last_move.clone(),
                           transposition_table,
                           nodes_searched, best_move, start_time,
                           /*EVAL_TOTAL_TIME, EVAL_TOTAL_RUNS, MOVES_TOTAL_TIME, MOVES_TOTAL_RUNS*/);

        // NMP fail-high
        if nmp >= beta {
            return beta;
        }
    }

    scored_moves.sort_by(|(_, score1), (_, score2)| score2.cmp(score1));

    let mut local_best_move = Direction::None;

    // println!();
    for (dir, _score) in scored_moves {
        board.move_snake(snake.clone(), dir.clone());

        let new_score = -minimax(board, enemy_id, snake_id, depth - 1, ply + 1, -beta, -alpha,
                                 should_nmp, dir.clone(), last_move.clone(),
                                 transposition_table,
                                 nodes_searched, best_move, start_time,
                                 // bestPath, searchedNodes,
                                 /*EVAL_TOTAL_TIME, EVAL_TOTAL_RUNS, MOVES_TOTAL_TIME, MOVES_TOTAL_RUNS*/);
        board.undo_move(snake.clone());

        if new_score > best_score {
            best_score = new_score;
            local_best_move = dir.clone();
            if ply == 0 {
                *best_move = dir.clone();
            }
        }
        if new_score > alpha {
            alpha = new_score;
            tt_flag = 3;
            if alpha >= beta {
                tt_flag = 2;
                break;
            }
        }
    }

    transposition_table[(board.zobrist_hash & 0x1FFFF) as usize] = TTEntry {
        zobrist: board.zobrist_hash.clone(),
        best_move: local_best_move.clone(),
        friendly_health: snake.borrow().health.clone(),
        enemy_health: enemy.borrow().health.clone(),
        depth: depth.clone(),
        score: best_score.clone(),
        flag: tt_flag,
        snake_head_x: snake_head.x.clone(),
        snake_head_y: snake_head.y.clone(),
        enemy_head_x: enemy_head.x.clone(),
        enemy_head_y: enemy_head.y.clone(),
    };

    return best_score;
}

pub fn write_eval_data(target_score: i32, board: GameBoard, snake_id: &str, enemy_id: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("/Users/ishaangoyal/RustroverProjects/starter-snake-rust/src/eval_data.csv").unwrap();

    if target_score.abs() > 1000 {
        return;
    }

    let snake = board.get_snake(snake_id);
    let enemy = board.get_snake(enemy_id);

    let snake_x = snake.borrow().head.x;
    let snake_y = snake.borrow().head.y;
    let enemy_x = enemy.borrow().head.x;
    let enemy_y = enemy.borrow().head.y;

    let mut bfs_term: i32 = 0;
    let mut health_term: i32 = snake.borrow().health.clone();
    let mut length_diff_term: i32 = snake.borrow().length.clone() - enemy.borrow().length.clone();

    let mut passability_matrix: [[bool; 11]; 11] = [[false; 11]; 11];

    for x in 0..board.width as usize {
        for y in 0..board.height as usize {
            if board.matrix[x][y] == CellContent::Empty || board.matrix[x][y] == CellContent::Food {
                passability_matrix[x][y] = true;
            }
        }
    }

    let snake_bfs = crate::eval::bfs(passability_matrix, snake_x as usize, snake_y as usize, 99);
    let enemy_bfs = crate::eval::bfs(passability_matrix, enemy_x as usize, enemy_y as usize, 99);

    for x in 0..board.width as usize {
        for y in 0..board.height as usize {
            if snake_bfs[x][y] != -1 {
                if snake_bfs[x][y] > enemy_bfs[x][y] {
                    bfs_term += 1;
                } else if enemy_bfs[x][y] > snake_bfs[x][y] {
                    bfs_term -= 1;
                }
            }
        }
    }



    let data = format!("{},{},{},{}", bfs_term, health_term, length_diff_term, target_score);
    writeln!(file, "{}", data).unwrap();
}