use crate::board::{CellContent, GameBoard};
use crate::{Battlesnake, Coord};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

pub fn eval(
    board: &GameBoard,
    snake: Rc<RefCell<Battlesnake>>,
    enemy: Rc<RefCell<Battlesnake>>,
) -> i32 {
    let me = snake.borrow().clone();
    let enemy = enemy.borrow().clone();
    let occupancy = board.boolboard.clone();
    let (my_area, other_area) = calculate_area(&me.head, &enemy.head, &occupancy);

    let area_diff = my_area - other_area;
    let health_diff = me.health - enemy.health;
    let norm_area_diff = (area_diff * 100) / 120;
    return norm_area_diff;
}

fn calculate_area(p1: &Coord, p2: &Coord, occupancy: &[[bool; 11]; 11]) -> (i32, i32) {
    let mut out = (0, 0);
    for x in 0..11 {
        for y in 0..11 {
            if occupancy[x][y] {
                continue;
            }
            let coord = &Coord {
                x: x as i32,
                y: y as i32,
            };
            let p1_dist = manhattan(p1, coord);
            let p2_dist = manhattan(p2, coord);
            if p1_dist > p2_dist {
                out.0 += 1;
            } else {
                out.1 += 1;
            }
        }
    }
    out
}

fn manhattan(p1: &Coord, p2: &Coord) -> i32 {
    return (p1.x - p2.x).abs() + (p1.y - p2.y).abs();
}
