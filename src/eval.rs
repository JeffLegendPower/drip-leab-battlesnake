// use std::cell::RefCell;
// use std::collections::VecDeque;
// use std::rc::Rc;
// use crate::Battlesnake;
// use crate::board::{CellContent, GameBoard};
//
// pub fn eval(board: &GameBoard, snake: Rc<RefCell<Battlesnake>>, enemy: Rc<RefCell<Battlesnake>>) -> i32 {
//     // let snake = board.get_snake(snake_id);
//     // let enemy = board.get_snake(enemy_id);
//
//     let mut score: i32 = 0;
//
//     // Quadratic score based on health to emphasize the danger of low health
//     // score -= enemy.health / 15;
//
//     let snake_x = snake.borrow().head.x;
//     let snake_y = snake.borrow().head.y;
//     let enemy_x = enemy.borrow().head.x;
//     let enemy_y = enemy.borrow().head.y;
//
//     // let mut passability_matrix: [[bool; 11]; 11] = [[false; 11]; 11];
//
//     // for x in 0..board.width as usize {
//     //     for y in 0..board.height as usize {
//     //         if board.matrix[x][y] == CellContent::Empty || board.matrix[x][y] == CellContent::Food {
//     //             passability_matrix[x][y] = true;
//     //         }
//     //     }
//     // }
//
//     let snake_bfs = bfs(&board, snake_x as usize, snake_y as usize);
//     let enemy_bfs = bfs(&board, enemy_x as usize, enemy_y as usize);
//
//     for x in 0..board.width as usize {
//         for y in 0..board.height as usize {
//             // the snake_bfs != -1 is just to make sure that the square is passable in the first place
//
//             // let dist = (x as i32 - board.width / 2).abs() + (y as i32 - board.height / 2).abs();
//             // let diff = if dist < (board.width - 1 / 2) {
//             //     2
//             // } else {
//             //     1
//             // };
//             if snake_bfs[x][y] != -1 {
//                 if snake_bfs[x][y] > enemy_bfs[x][y] {
//                     score += 1;
//                 } else if enemy_bfs[x][y] > snake_bfs[x][y] {
//                     score -= 1;
//                 }
//             }
//         }
//         // println!();
//     }
//
//     // score /= 2;
//
//     // if snake.borrow().health < 70 {
//     //     score += (snake.borrow().health / 15) as i32 * (snake.borrow().health / 15) as i32 - 20;
//     // }
//     score += (snake.borrow().length - enemy.borrow().length) as i32 * 2;
//
//     // // TODO we can also like subtract number of enemy legal moves
//     // // TODO also we can account for who has more squares available to travel to
//     // score += board.generate_possible_moves(snake.clone()).len() as i32 * 3;
//     // score -= board.generate_possible_moves(snake.clone()).len() as i32 * 3;
//     // score += (snake.borrow().length - enemy.borrow().length) * 2;
//     //
//     // // Let's avoid the wall as we can easily die there
//     // let dist_to_wall = snake.borrow().head.x.abs().min((snake.borrow().head.x - board.width).abs()).min(
//     //     snake.borrow().head.y.abs().min((snake.borrow().head.y - board.height).abs()));
//     //
//     // let enemy_dist_to_wall = enemy.borrow().head.x.abs().min((enemy.borrow().head.x - board.width).abs()).min(
//     //     enemy.borrow().head.y.abs().min((enemy.borrow().head.y - board.height).abs()));
//     //
//     // score += (dist_to_wall * dist_to_wall) - (enemy_dist_to_wall * enemy_dist_to_wall) / 3;
//
//     // println!("{}", snake.borrow().name);
//     return score;
// }
//
// pub(crate) fn bfs(board: &GameBoard, start_x: usize, start_y: usize) -> [[i32; 11]; 11] {
//     let mut distances: [[i32; 11]; 11] = [[-1; 11]; 11];
//     let mut queue = VecDeque::new();
//
//     // Cardinal directions
//     let directions = vec![(0, 1), (0, -1), (1, 0), (-1, 0)];
//
//     // Start with initial point
//     queue.push_back((start_x, start_y));
//     distances[start_x][start_y] = 0;
//
//     // Perform BFS
//     while let Some((x, y)) = queue.pop_front() {
//         for (dx, dy) in &directions {
//             let new_x = x as i32 + dx;
//             let new_y = y as i32 + dy;
//
//             // Check if new position is within bounds
//             if new_x >= 0 && new_x < 11i32 && new_y >= 0 && new_y < 11i32 {
//                 let new_x = new_x as usize;
//                 let new_y = new_y as usize;
//
//                 // Check if the cell is passable and not visited
//                 if (board.matrix[new_x][new_y] == CellContent::Empty
//                     || board.matrix[new_x][new_y] == CellContent::Food)
//                     && distances[new_x][new_y] == -1 {
//                     distances[new_x][new_y] = distances[x][y] + 1;
//                     queue.push_back((new_x, new_y));
//                 }
//             }
//         }
//     }
//
//     distances
// }
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use crate::Battlesnake;
use crate::board::{CellContent, Direction, GameBoard};

pub fn eval(board: &GameBoard, snake: Rc<RefCell<Battlesnake>>, enemy: Rc<RefCell<Battlesnake>>) -> i32 {
    // let snake = board.get_snake(snake_id);
    // let enemy = board.get_snake(enemy_id);

    let mut score: i32 = 0;

    // Quadratic score based on health to emphasize the danger of low health
    // score -= enemy.health / 15;

    let snake_x = snake.borrow().head.x;
    let snake_y = snake.borrow().head.y;
    let enemy_x = enemy.borrow().head.x;
    let enemy_y = enemy.borrow().head.y;

    let snake_length = snake.borrow().length;
    let enemy_length = enemy.borrow().length;

    let mut passability_matrix: [[bool; 11]; 11] = [[false; 11]; 11];

    for x in 0..board.width as usize {
        for y in 0..board.height as usize {
            if board.matrix[x][y] == CellContent::Empty || board.matrix[x][y] == CellContent::Food {
                passability_matrix[x][y] = true;
            }
        }
    }


    let (snake_bfs, enemy_bfs) = bfs(&passability_matrix,
                                       snake_x as usize, snake_y as usize,
                                       enemy_x as usize, enemy_y as usize);

    let mut enemys = 0;
    for x in 0..board.width as usize {
        for y in 0..board.height as usize {
            // the snake_bfs != -1 is just to make sure that the square is passable in the first place

            // let dist = (x as i32 - board.width / 2).abs() + (y as i32 - board.height / 2).abs();
            // let diff = if dist < (board.width - 1 / 2) {
            //     2
            // } else {
            //     1
            // };
            // println!("{}, {}", snake_bfs[x][y], enemy_bfs[x][y]);
            // if enemy_bfs[x][y] != -1 && snake_bfs[x][y] != -1 {
            if passability_matrix[x][y] {
                enemys += 1;

                // let diff = if (5 - x as i32).abs() + (5 - y as i32).abs() < 5 {
                //     125
                // } else {
                //     100
                // };
                let diff = 100;

                if snake_bfs[x][y] < enemy_bfs[x][y] {
                    score += diff;
                } else if enemy_bfs[x][y] < snake_bfs[x][y] {
                    score -= diff;
                } else if snake_length > enemy_length {
                    score += diff;
                } else if enemy_length > snake_length {
                    score -= diff;
                }
                // if enemy_bfs[x][y] > snake_bfs[x][y] {
                //     score += 1;
                // } else if snake_bfs[x][y] > enemy_bfs[x][y] {
                //     println!("E");
                //     score -= 1;
                // }
            }
        }
        // println!();
    }

    // println!("Enemy squares: {}", enemys);

    // score /= 2;
    // score = -score;

    if snake.borrow().health < 70 {
        score += 100 * (snake.borrow().health / 12) * (snake.borrow().health / 12) - 2000;

    }
    score += (snake.borrow().length - enemy.borrow().length) * 300;
    //
    // if print {
    //     println!("Length score: {}", (snake.borrow().length - enemy.borrow().length) * 3);
    // }

    // // TODO we can also like subtract number of enemy legal moves
    // // TODO also we can account for who has more squares available to travel to
    // score += board.generate_possible_moves(snake.clone()).len() as i32 * 3;
    // score -= board.generate_possible_moves(snake.clone()).len() as i32 * 3;
    // score += (snake.borrow().length - enemy.borrow().length) * 2;
    //
    // // Let's avoid the wall as we can easily die there
    // let dist_to_wall = snake.borrow().head.x.abs().min((snake.borrow().head.x - board.width).abs()).min(
    //     snake.borrow().head.y.abs().min((snake.borrow().head.y - board.height).abs()));
    //
    // let enemy_dist_to_wall = enemy.borrow().head.x.abs().min((enemy.borrow().head.x - board.width).abs()).min(
    //     enemy.borrow().head.y.abs().min((enemy.borrow().head.y - board.height).abs()));
    //
    // score += (dist_to_wall * dist_to_wall) - (enemy_dist_to_wall * enemy_dist_to_wall) / 3;

    // println!("{}", snake.borrow().name);
    return score;
}

pub(crate) fn bfs(passability_matrix: &[[bool; 11]; 11],
                    start_x_1: usize, start_y_1: usize,
                    start_x_2: usize, start_y_2: usize) -> ([[i32; 11]; 11], [[i32; 11]; 11]) {
    let mut distances_1: [[i32; 11]; 11] = [[9999; 11]; 11];
    let mut distances_2: [[i32; 11]; 11] = [[9999; 11]; 11];
    let mut queue_1 = VecDeque::new();
    let mut queue_2 = VecDeque::new();

    // Cardinal directions
    let directions = vec![(0, 1), (0, -1), (1, 0), (-1, 0)];

    // Start with initial point
    queue_1.push_back((start_x_1, start_y_1));
    distances_1[start_x_1][start_y_1] = 0;

    queue_2.push_back((start_x_2, start_y_2));
    distances_2[start_x_2][start_y_2] = 0;

    while let Some((x_1, y_1)) = queue_1.pop_front() {
        for (dx, dy) in &directions {
            let new_x_1 = (x_1 as i32 + dx) as usize;
            let new_y_1 = (y_1 as i32 + dy) as usize;

            // Check if new position is within bounds
            if new_x_1 < 11 && new_y_1 < 11 {
                // Check if the cell is passable and not visited
                if passability_matrix[new_x_1][new_y_1] && distances_1[new_x_1][new_y_1] == 9999 {
                    distances_1[new_x_1][new_y_1] = distances_1[x_1][y_1] + 1;

                    if distances_1[new_x_1][new_y_1] <= distances_2[new_x_1][new_y_1] {
                        queue_1.push_back((new_x_1, new_y_1));
                    }
                }
            }

            if let Some((x_2, y_2)) = queue_2.pop_front() {
                for (dx, dy) in &directions {
                    let new_x_2 = (x_2 as i32 + dx) as usize;
                    let new_y_2 = (y_2 as i32 + dy) as usize;

                    if new_x_2 < 11 && new_y_2 < 11 {
                        // Check if the cell is passable and not visited
                        if passability_matrix[new_x_2][new_y_2] && distances_2[new_x_2][new_y_2] == 9999 {
                            distances_2[new_x_2][new_y_2] = distances_2[x_2][y_2] + 1;

                            if distances_2[new_x_2][new_y_2] <= distances_1[new_x_2][new_y_2] {
                                queue_2.push_back((new_x_2, new_y_2));
                            }
                        }
                    }
                }
            }
        }
    }

    while let Some((x_2, y_2)) = queue_2.pop_front() {
        for (dx, dy) in &directions {
            let new_x_2 = (x_2 as i32 + dx) as usize;
            let new_y_2 = (y_2 as i32 + dy) as usize;

            if new_x_2 < 11 && new_y_2 < 11 {
                // Check if the cell is passable and not visited
                if passability_matrix[new_x_2][new_y_2] && distances_2[new_x_2][new_y_2] == 9999 {
                    distances_2[new_x_2][new_y_2] = distances_2[x_2][y_2] + 1;

                    if distances_2[new_x_2][new_y_2] <= distances_1[new_x_2][new_y_2] {
                        queue_2.push_back((new_x_2, new_y_2));
                    }
                }
            }
        }
    }

    (distances_1, distances_2)
}