use std::cell::RefCell;
use std::cmp::Ordering;
use std::fs::OpenOptions;
use std::time::Instant;
use std::io::Write;
use std::rc::Rc;
use crate::{Battlesnake, Coord};
use crate::board::{CellContent, Direction, GameBoard};
use crate::eval::{bfs, eval};
use crate::game_recorder::{GameRecorder, RoundInfo};
use crate::transposition_table::TTEntry;

pub fn think(
    game_recorder: &mut GameRecorder,
    mut board: GameBoard, snake: Battlesnake, transposition_table: &mut Vec<TTEntry>, killers: &mut [(Coord, Coord); 1000]
) -> Direction {
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

    // println!("Eval: {}", eval(&board, board.get_snake(&enemy_id).clone(), ref_snake.clone(), false));
    drop(enemy_borrow);

    // let mut depth = 1;
    let mut depth = 0;
    let mut score = 0;
    let mut path: Vec<Direction> = Vec::new();

    let mut singular = 0;
    let mut not_singular = 0;

    let mut history = [[[[0; 11]; 11]; 11]; 11];

    // println!("Before Endgame: {}", before_endgame(&board, ref_snake.clone(), board.get_snake(&enemy_id).clone()));

    // while start_time.elapsed().as_millis() < /*250*/10 && depth <= 50 {
    while depth <= 9 {
        depth += 1;

        singular = 0;
        not_singular = 0;

        let mut temp_best_move = Direction::None;
        score = minimax(&mut board, &snake.id, &enemy_id, depth.clone(), 0, -9999999, 9999999,
                        true,
                        transposition_table,
                        &mut nodes_searched, &mut temp_best_move, start_time,
                        &mut path,
                        &mut history, killers,
                        &mut singular, &mut not_singular
        );

        // If the search cuts early, it will be bad, so we will prevent that from affecting the best move
        // if start_time.elapsed().as_millis() <= /*290*/12 && temp_best_move != Direction::None {
        if temp_best_move != Direction::None {

            best_move = temp_best_move.clone();
        }
    }

    // path.reverse();
    println!("Nodes Searched: {}, Depth {}, Best Score: {}, Best Direction: {:?}", nodes_searched, depth, score, best_move);
    println!("Ordered Ratio: {}", singular as f64 / (singular + not_singular + 1) as f64);

    // println!("NPS: {}", nodes_searched as f64 / start_time.elapsed().as_secs_f64());
    // println!("Best Path: {:?}", path);

    // if score.abs() < 50000 {
        record_round(score, board.clone(), &snake.id, &enemy_id, game_recorder);
    // }

    // let mut file = OpenOptions::new()
    //     .write(true)
    //     .append(true)
    //     .create(true)
    //     .open("/Users/ishaangoyal/RustroverProjects/starter-snake-rust/prunedata/prune_data_1.csv").unwrap();
    //
    // writeln!(file, "{}", nodes_searched).unwrap();

    return best_move;
}

pub fn minimax(board: &mut GameBoard, snake_id: &str, enemy_id: &str, mut depth: i32, ply: i32, mut alpha: i32, mut beta: i32,
               should_nmp: bool,
               transposition_table: &mut Vec<TTEntry>,
               nodes_searched: &mut i32, best_move: &mut Direction, start_time: Instant, past_moves: &mut Vec<Direction>,
               history: &mut [[[[i32; 11]; 11]; 11]; 11], killers: &mut [(Coord, Coord); 1000],
               singular: &mut i32, not_singular: &mut i32) -> i32 {
    *nodes_searched += 1;


    let snake = board.get_snake(snake_id).clone();
    let enemy = board.get_snake(enemy_id).clone();

    let possible_moves = board.generate_possible_moves(snake.clone());

    // Dead
    if possible_moves.is_empty() || snake.borrow().health <= 0 {
        return -100000 + ply;
    }

    // Potential game-ending branch
    if possible_moves.len() == 1 {
        depth += 1;
    }

    // if depth <= 0 || start_time.elapsed().as_millis() >= /*300*/15 {
    if depth <= 0 {
        return -eval(board, enemy.clone(), snake.clone());
    }

    let mut best_score = -999999;

    let snake_head = snake.borrow().head.clone();
    let enemy_head = enemy.borrow().head.clone();

    let entry = &transposition_table[(board.zobrist_hash & 0x7FFFF) as usize];
    let tt_hit = entry.zobrist == board.zobrist_hash
        && entry.friendly_health == snake.borrow().health
        && entry.enemy_health == enemy.borrow().health
        && entry.snake_head_x == snake_head.x
        && entry.snake_head_y == snake_head.y
        && entry.enemy_head_x == enemy_head.x
        && entry.enemy_head_y == enemy_head.y;
        // && entry.board_hash == board.board_hash;

    if ply > 0 && tt_hit && entry.depth >= depth {
        match entry.flag {
            1 => {
                alpha = alpha.max(entry.score);
            }
            2 => {
                beta = beta.min(entry.score);
            }
            _ => {
                // alpha = alpha.max(entry.score);
                // beta = beta.min(entry.score);
                return entry.score;
            }
        }
        if alpha >= beta {
            return entry.score;
        }
    }

    let mut tt_flag = 1;

    let mut scored_moves: Vec<(Direction, i32)> = possible_moves.iter()
        .map(|dir| (*dir,
            // NO TONKAs, no AUs, no NEWTONMETERS, no INVERSEKILOJOULESPERMETERSSQUARED, no GOLDMAN, 6 ON AP PHYSICS ABCD
            if tt_hit && dir == &entry.best_move {
                100_000_000
            } else if tt_hit && dir == &entry.second_best_move {
                90_000_000
            } else if tt_hit && dir == &entry.worst_move {
                -100_000_000
            } else {
                let new_head = move_coord(&snake_head, dir);

                if killers[ply as usize].0 == snake_head && killers[ply as usize].1 == new_head {
                    10_000_000
                } else if history[snake_head.x as usize][snake_head.y as usize][new_head.x as usize][new_head.y as usize] > 0 {
                    history[snake_head.x as usize][snake_head.y as usize][new_head.x as usize][new_head.y as usize] * 1000
                } else if past_moves.len() >= 4
                    && dir == &past_moves[past_moves.len() - 2]
                    && dir == &past_moves[past_moves.len() - 4] { // Tempo bonus
                    1_000
                } else {
                    let center_distance = (new_head.x - 5).abs() + (new_head.y - 5).abs();
                    let center_distance_enemy = (enemy_head.x - 5).abs() + (enemy_head.y - 5).abs();

                    let mut num_adj_occupied = 0;
                    if new_head.x == 10 || board.matrix[(new_head.x + 1) as usize][new_head.y as usize] != CellContent::Empty {
                        num_adj_occupied += 100;
                    }
                    if new_head.y == 10 || board.matrix[new_head.x as usize][(new_head.y + 1) as usize] != CellContent::Empty {
                        num_adj_occupied += 100;
                    }
                    if new_head.x == 0 || board.matrix[(new_head.x - 1) as usize][new_head.y as usize] != CellContent::Empty {
                        num_adj_occupied += 100;
                    }
                    if new_head.y == 0 || board.matrix[new_head.x as usize][(new_head.y - 1) as usize] != CellContent::Empty {
                        num_adj_occupied += 100;
                    }

                    if board.matrix[new_head.x as usize][new_head.y as usize] == CellContent::Food {
                        -1_000_000 + 50_000 - num_adj_occupied + (center_distance_enemy - center_distance)
                    } else {
                        -1_000_000 + -num_adj_occupied + (center_distance_enemy - center_distance)
                    }
                }
            }
    )).collect();

    // Null Move Pruning
    if depth > 5 && should_nmp {
        past_moves.push(Direction::None);

        // Give the enemy snake an extra move, if we are still doing better, then this is a great position
        let nmp = -minimax(board, enemy_id, snake_id, 3, ply + 1, -beta, -alpha,
                           false,
                           transposition_table,
                           nodes_searched, best_move, start_time,
                           past_moves,
                           history, killers,
                           singular, not_singular);

        past_moves.pop();
        // NMP fail-high
        if nmp >= beta {
            return beta;
        }
    }

    match scored_moves.len() {
        2 => sort_2(&mut scored_moves),
        3 => sort_3(&mut scored_moves),
        _ => {}
    }

    let mut local_best_move = Direction::None;
    let mut second_local_best_move = Direction::None;
    let mut worst_local_move = Direction::None;

    let mut worst_score = 999999;

    // println!();
    let mut i = 0;
    for (dir, _score) in &scored_moves {
        // let redux = if i > 1 { 1 } else { 0 };
        // let null_window = if i > 1 { true } else { false };
        let null_window = false;

        board.move_snake(snake.clone(), dir.clone());

        past_moves.push(dir.clone());

        let new_score = if i <= 1 {
            -minimax(board, enemy_id, snake_id, depth - 1, ply + 1,
                     if null_window { -alpha - 1 } else { -beta }, -alpha,
                     should_nmp,
                     transposition_table,
                     nodes_searched, best_move, start_time,
                     past_moves,
                     history, killers,
                     singular, not_singular)
        } else {
            let temp_score = -minimax(board, enemy_id, snake_id, depth - 1 - 2, ply + 1,
                                      -alpha - 1, -alpha,
                                      should_nmp,
                                      transposition_table,
                                      nodes_searched, best_move, start_time,
                                      past_moves,
                                      history, killers,
                                      singular, not_singular);

            if temp_score > alpha {
                -minimax(board, enemy_id, snake_id, depth - 1, ply + 1,
                         -beta, -alpha,
                         should_nmp,
                         transposition_table,
                         nodes_searched, best_move, start_time,
                         past_moves,
                         history, killers,
                         singular, not_singular)
            } else {
                temp_score
            }
        };



        board.undo_move(snake.clone());

        past_moves.pop();

        if new_score > best_score {
            best_score = new_score;
            second_local_best_move = local_best_move.clone();
            local_best_move = dir.clone();
            if ply == 0 {
                *best_move = dir.clone();
            }
        }
        if new_score > alpha {
            alpha = new_score;
            tt_flag = 3;
            if alpha >= beta {
                let new_head = move_coord(&snake_head, dir);
                history[snake_head.x as usize][snake_head.y as usize][new_head.x as usize][new_head.y as usize] += depth * depth;
                killers[ply as usize] = (snake_head, new_head);
                tt_flag = 2;
                break;
            }
        }
        if new_score < worst_score {
            worst_score = new_score;
            worst_local_move = dir.clone();
        }

        i += 1;
    }

    // if local_best_move == scored_moves[0].0.clone() {
    //     *num_ordered += 1;
    // } else {
    //     *num_unordered += 1
    // }
    if scored_moves.len() == 1 {
        *singular += 1;
    } else {
        *not_singular += 1;
    }

    transposition_table[(board.zobrist_hash & 0x7FFFF) as usize] = TTEntry {
        zobrist: board.zobrist_hash.clone(),
        best_move: local_best_move.clone(),
        second_best_move: second_local_best_move.clone(),
        worst_move: worst_local_move.clone(),
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

fn sort_2(arr: &mut Vec<(Direction, i32)>) {
    if arr[1].1 > arr[0].1 {
        arr.swap(0, 1);
    }
}

fn sort_3(arr: &mut Vec<(Direction, i32)>) {
    if arr[1].1 > arr[0].1 {
        arr.swap(0, 1);
    }
    if arr[2].1 > arr[1].1 {
        arr.swap(1, 2);
    }
    if arr[1].1 > arr[0].1 {
        arr.swap(0, 1);
    }
}

fn move_coord(coord: &Coord, dir: &Direction) -> Coord {
    match dir {
        Direction::Up => Coord {
            x: coord.x,
            y: coord.y + 1,
        },
        Direction::Down => Coord {
            x: coord.x,
            y: coord.y - 1,
        },
        Direction::Left => Coord {
            x: coord.x - 1,
            y: coord.y,
        },
        Direction::Right => Coord {
            x: coord.x + 1,
            y: coord.y,
        },
        Direction::None => Coord {
            x: coord.x,
            y: coord.y,
        },
    }
}

fn before_endgame(board: &GameBoard, snake: Rc<RefCell<Battlesnake>>, enemy: Rc<RefCell<Battlesnake>>) -> bool {
    return (snake.borrow().length + enemy.borrow().length) > (board.width * board.height) / 4;
}

pub fn record_round(target_score: i32, board: GameBoard, snake_id: &str, enemy_id: &str, game_recorder: &mut GameRecorder) {
    // let mut file = OpenOptions::new()
    //     .write(true)
    //     .append(true)
    //     .create(true)
    //     .open("/Users/ishaangoyal/RustroverProjects/starter-snake-rust/src/eval_data.csv").unwrap();

    if target_score > 50000 {
        game_recorder.ending = 1.0;
        return;
    } else if target_score < -50000 {
        game_recorder.ending = 0.0;
        return;
    }

    let snake = board.get_snake(snake_id);
    let enemy = board.get_snake(enemy_id);

    let snake_x = snake.borrow().head.x;
    let snake_y = snake.borrow().head.y;
    let enemy_x = enemy.borrow().head.x;
    let enemy_y = enemy.borrow().head.y;

    let snake_length = snake.borrow().length;
    let enemy_length = enemy.borrow().length;

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

    // let snake_bfs = crate::eval::bfs(&passability_matrix, snake_x as usize, snake_y as usize);
    // let enemy_bfs = crate::eval::bfs(&passability_matrix, enemy_x as usize, enemy_y as usize);

    let (snake_bfs, enemy_bfs) = bfs(&passability_matrix,
                                     snake_x as usize, snake_y as usize,
                                     enemy_x as usize, enemy_y as usize);
    let mut bfs_snake = 0;
    let mut bfs_enemy = 0;

    for x in 0..board.width as usize {
        for y in 0..board.height as usize {
            if passability_matrix[x][y] {
                if snake_bfs[x][y] < enemy_bfs[x][y] {
                    bfs_snake += 1;
                } else if enemy_bfs[x][y] < snake_bfs[x][y] {
                    bfs_enemy += 1;
                } else if snake_length > enemy_length {
                    bfs_snake += 1;
                } else if enemy_length > snake_length {
                    bfs_enemy += 1;
                }
            }
        }
        // println!();
    }

    game_recorder.rounds.push(RoundInfo {
        snake_bfs: bfs_snake,
        enemy_bfs: bfs_enemy,
        snake_length,
        enemy_length,
        health: health_term,
    });

}