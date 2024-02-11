use std::fs::{File, OpenOptions};
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use crate::{Battlesnake, Coord};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellContent {
    Food,
    Snake(String),
    Hazard,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub(crate) snake_id: String,
    pub(crate) direction: Direction,
    pub(crate) oldTail: Coord,
    // pub(crate) secondLastTail: Coord,
    pub(crate) newHead: Coord,
    pub(crate) oldHead: Coord,
    pub(crate) ateFood: bool,
    pub(crate) oldHealth: i32,
    pub(crate) fail: bool, // half-assed movegen bandaid
}

#[derive(Debug, Clone)]
pub struct GameBoard {
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) matrix: Vec<Vec<CellContent>>,
    pub(crate) snakes: Vec<Rc<RefCell<Battlesnake>>>,
    pub(crate) food: Vec<Coord>,
    // pub(crate) history: Vec<(Vec<Vec<CellContent>>, Vec<Rc<RefCell<Battlesnake>>>, Vec<Coord>)>, // Added history field
    pub(crate) history: Vec<Action>,
}

impl GameBoard {
    pub fn new(width: i32, height: i32, food: Vec<Coord>, snakes: Vec<Battlesnake>, hazards: Vec<Coord>) -> Self {
        let mut matrix = vec![vec![CellContent::Empty; height as usize]; width as usize];

        for coord in &food {
            matrix[coord.x as usize][coord.y as usize] = CellContent::Food;
        }

        let mut bitmap: u128 = 0;

        let mut ref_snakes = Vec::new();
        for snake in snakes {
            let ref_snake = Rc::new(RefCell::new(snake.clone()));
            ref_snakes.push(ref_snake.clone());
            for coord in &snake.body {
                matrix[coord.x as usize][coord.y as usize] = CellContent::Snake(snake.id.clone());
                bitmap |= 1 << (coord.x * 11 + coord.y);
            }
        }

        for coord in hazards {
            matrix[coord.x as usize][coord.y as usize] = CellContent::Hazard;
        }

        Self {
            width,
            height,
            matrix,
            snakes: ref_snakes,
            food,
            history: Vec::new(), // Initialize history
        }
    }

    pub fn move_snake(&mut self, snake: Rc<RefCell<Battlesnake>>, direction: Direction) -> bool {
        let mut borrow = snake.borrow_mut();

        let mut action: Action = Action {
            snake_id: borrow.id.clone(),
            direction: direction.clone(),
            oldTail: Coord { x: 0, y: 0 },
            // secondLastTail: Coord { x: 0, y: 0 },
            newHead: Coord { x: 0, y: 0 },
            oldHead: Coord { x: 0, y: 0 },
            ateFood: false,
            oldHealth: borrow.health.clone(),
            fail: false,
        };;

        let old_head = borrow.head.clone();
        action.oldHead = old_head.clone();

        let new_head = match direction {
            Direction::Up => Coord { x: old_head.clone().x, y: old_head.clone().y + 1 },
            Direction::Down => Coord { x: old_head.clone().x, y: old_head.clone().y - 1 },
            Direction::Left => Coord { x: old_head.clone().x - 1, y: old_head.clone().y },
            Direction::Right => Coord { x: old_head.clone().x + 1, y: old_head.clone().y },
        };

        if new_head.x >= self.width || new_head.x < 0 || new_head.y >= self.height || new_head.y < 0 {
            //     let mut file = OpenOptions::new()
            //         .write(true)
            //         .append(true)
            //         .create(true)
            //         .open("/Users/ishaangoyal/RustroverProjects/starter-snake-rust/src/output.txt").unwrap();
            //     // let mut file = File::create("/Users/ishaangoyal/RustroverProjects/starter-snake-rust/src/output.txt").unwrap();
            //
            //     let msg = format!("ERROR e: {:?}, f: {:?}\n", old_head.clone(), new_head.clone());
            //     writeln!(file, "{}", msg);
            action.fail = true;
            self.history.push(action.clone());
            return false;
        }
        action.newHead = new_head.clone();

        // Add new head to the snake body
        borrow.body.insert(0, new_head.clone());
        borrow.head = new_head.clone();

        // Remove the tail of the snake
        let old_tail = borrow.body.pop().unwrap();
        action.oldTail = old_tail.clone();

        // println!("a: {:?}, b: {:?}, c: {}", new_head, old_tail, borrow.name);
        // println!("a: {:?}", old_head);

        borrow.health -= 1;

        // Check if snake stepped on food
        if self.matrix[new_head.x as usize][new_head.y as usize] == CellContent::Food {
            borrow.health = 100; // Refill health
            borrow.body.push(old_tail.clone()); // Make the snake longer fatass
            borrow.length += 1;
            action.ateFood = true;
        } else {
            action.ateFood = false;
        }

        // no worries
        // if snake.health <= 0 {
        //     // Remove the dead snake from the game board
        //     for coord in &snake.body {
        //         self.matrix[coord.x as usize][coord.y as usize] = CellContent::Empty;
        //     }
        //     // self.snakes.retain(|s| s.id != snake.id);
        // }

        // TODO handle hazards

        // Update the game board
        // removing the old tail MUST GO FIRST
        self.matrix[old_tail.x as usize][old_tail.y as usize] = CellContent::Empty;
        self.matrix[new_head.x as usize][new_head.y as usize] = CellContent::Snake(borrow.id.clone());

        // action.secondLastTail = borrow.body.last().unwrap().clone();

        self.history.push(action.clone());

        // let mut file = OpenOptions::new()
        //     .write(true)
        //     .append(true)
        //     .create(true)
        //     .open("/Users/ishaangoyal/RustroverProjects/starter-snake-rust/src/output.txt").unwrap();
        // // let mut file = File::create("/Users/ishaangoyal/RustroverProjects/starter-snake-rust/src/output.txt").unwrap();
        //
        // let msg = format!("c: {:?}, d: {:?}\n", old_head.clone(), new_head.clone());
        // writeln!(file, "{}", msg);
        return true;
    }

    pub fn undo_move(&mut self) {
        // if let Some((matrix, snakes, food)) = self.history.pop() {
        //     self.matrix = matrix;
        //     self.snakes = snakes;
        //     self.food = food;
        // }

        if let Some(action) = self.history.pop() {
            if action.fail {
                return;
            }
            let mut snake = self.get_snake(&action.snake_id).clone();
            let mut borrow = snake.borrow_mut();

            if action.ateFood {
                self.matrix[action.newHead.x as usize][action.newHead.y as usize] = CellContent::Food;
                borrow.length -= 1;
            } else {
                self.matrix[action.newHead.x as usize][action.newHead.y as usize] = CellContent::Empty;
                borrow.body.push(action.oldTail.clone());
            }
            self.matrix[action.oldTail.x as usize][action.oldTail.y as usize] = CellContent::Snake(borrow.id.clone());


            borrow.health = action.oldHealth.clone();
            borrow.body.remove(0);
            // print!("a: {:?}", borrow.head);
            borrow.head = action.oldHead.clone();
            // println!("b: {:?}", borrow.head);
        }
    }

    pub fn generate_possible_moves(&self, snake: Rc<RefCell<Battlesnake>>) -> Vec<Direction> {
        // let mut file = OpenOptions::new()
        //     .write(true)
        //     .append(true)
        //     .create(true)
        //     .open("/Users/ishaangoyal/RustroverProjects/starter-snake-rust/src/output.txt").unwrap();

        let mut possible_moves = Vec::new();
        let mut last_ditch_moves = Vec::new();

        let directions = [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];

        let old_head = snake.borrow().head.clone();
        for direction in directions.iter() {
            let new_head = match direction {
                Direction::Up => Coord { x: old_head.x.clone(), y: old_head.y.clone() + 1 },
                Direction::Down => Coord { x: old_head.x.clone(), y: old_head.y.clone() - 1 },
                Direction::Left => Coord { x: old_head.x.clone() - 1, y: old_head.y.clone() },
                Direction::Right => Coord { x: old_head.x.clone() + 1, y: old_head.y.clone() },
            };

            // Check if the new head position is within the game board boundaries
            if new_head.x < 0 || new_head.x >= self.width || new_head.y < 0 || new_head.y >= self.height {
                continue;
            }

            // Check if the new head position does not collide with the snake itself or any other snake
            // TODO a possible error will be a snake might just gobble up an immobile snake (another snake on the board which isn't part of the minimax)
            if let CellContent::Snake(ref other_snake_id) = self.matrix[new_head.x as usize][new_head.y as usize] {
                let other_snake = self.get_snake(other_snake_id);
                // if snake == other_snake || self.snakes.iter().any(|s| s.body.contains(&new_head)) {
                // Can ignore the tail because that gets moved forward
                // if self.snakes.iter().take(self.snakes.len()).any(|s| s.borrow().body.contains(&new_head)) {
                //     continue;
                // }
                if snake.borrow().id == other_snake.borrow().id {
                    if snake.borrow().body.iter().take((snake.borrow().length - 1) as usize).any(|x| x == &new_head) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // NOTE this MUST go after body collision checks otherwise last ditch moves might be populated with illegal moves
            let mut head_to_head = false;
            for enemy_snake in &self.snakes {
                if enemy_snake.borrow().id == snake.borrow().id {
                    continue;
                }
                if (enemy_snake.borrow().head.x - new_head.x) * (enemy_snake.borrow().head.x - new_head.x)
                    + (enemy_snake.borrow().head.y - new_head.y) * (enemy_snake.borrow().head.y - new_head.y) <= 1
                    && snake.borrow().length <= enemy_snake.borrow().length {
                    // continue the outer loop
                    head_to_head = true;
                    if snake.borrow().length == enemy_snake.borrow().length {
                        last_ditch_moves.push(direction.clone());
                    }
                    break;
                }
            }
            if head_to_head {
                continue;
            }

            possible_moves.push(direction.clone());
        }

        if possible_moves.is_empty() {
            // if send {
            //     let msg = format!("a: {:?}, b: {:?}, c: {:?}", old_head.clone(), last_ditch_moves.clone(), snake.borrow().name.clone());
            //     writeln!(file, "{}", msg).unwrap();
            //
            //     let goodsnake = self.snakes.iter().find(|s| s.borrow().name == "Rust Starter Project").unwrap();
            //     let badsnake = self.snakes.iter().find(|s| s.borrow().name == "Snake2").unwrap();
            //
            //     for y in (0usize..(self.height as usize)).rev() {
            //         for x in 0..self.width as usize {
            //             let msg = if self.matrix[x][y] == CellContent::Empty {
            //                 "."
            //             } else if self.matrix[x][y] == CellContent::Food {
            //                 "f"
            //             } else if self.matrix[x][y] == CellContent::Snake(goodsnake.borrow().id.clone()) {
            //                 if goodsnake.borrow().head.x == (x as i32) && goodsnake.borrow().head.y == (y as i32) {
            //                     "X"
            //                 } else {
            //                     "x"
            //                 }
            //             } else {
            //                 if badsnake.borrow().head.x == (x as i32) && badsnake.borrow().head.y == (y as i32) {
            //                     "O"
            //                 } else {
            //                     "o"
            //                 }
            //             };
            //             write!(file, "{} ", msg).unwrap();
            //         }
            //         writeln!(file).unwrap();
            //     }
            //     writeln!(file).unwrap();
            // }

            return last_ditch_moves.clone();
        }

        // if send {
        //     let msg = format!("a: {:?}, b: {:?}, c: {:?}", old_head.clone(), possible_moves.clone(), snake.borrow().name.clone());
        //     writeln!(file, "{}", msg).unwrap();
        //
        //     let goodsnake = self.snakes.iter().find(|s| s.borrow().name == "Rust Starter Project").unwrap();
        //     let badsnake = self.snakes.iter().find(|s| s.borrow().name == "Snake2").unwrap();
        //
        //     for y in (0usize..(self.height as usize)).rev() {
        //         for x in 0..self.width as usize {
        //             let msg = if self.matrix[x][y] == CellContent::Empty {
        //                 "."
        //             } else if self.matrix[x][y] == CellContent::Food {
        //                 "f"
        //             } else if self.matrix[x][y] == CellContent::Snake(goodsnake.borrow().id.clone()) {
        //                 if goodsnake.borrow().head.x == (x as i32) && goodsnake.borrow().head.y == (y as i32) {
        //                     "X"
        //                 } else {
        //                     "x"
        //                 }
        //             } else {
        //                 if badsnake.borrow().head.x == (x as i32) && badsnake.borrow().head.y == (y as i32) {
        //                     "O"
        //                 } else {
        //                     "o"
        //                 }
        //             };
        //             write!(file, "{} ", msg).unwrap();
        //         }
        //         writeln!(file).unwrap();
        //     }
        //     writeln!(file).unwrap();
        // }

        // file.flush();
        possible_moves.clone()
    }

    pub fn get_snake(&self, snake_id: &str) -> &Rc<RefCell<Battlesnake>> {
        self.snakes.iter().find(|s| s.borrow().id == snake_id).unwrap()
    }

    pub fn clone(&self) -> Self {
        let matrix = self.matrix.clone();
        let width = self.width;
        let height = self.height;
        let food = self.food.clone();

        let snakes = self.snakes.iter().map(|snake| {
            let snake_clone = snake.borrow().clone();
            Rc::new(RefCell::new(snake_clone))
        }).collect();

        let history = self.history.clone();

        Self {
            width,
            height,
            matrix,
            snakes,
            food,
            history,
        }
    }
}