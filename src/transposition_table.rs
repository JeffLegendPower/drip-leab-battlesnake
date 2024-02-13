use crate::board::Direction;

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub(crate) zobrist: u64,
    pub(crate) best_move: Direction,

    // A bit of a hack to reduce collisions
    pub(crate) friendly_health: i32,
    pub(crate) enemy_health: i32,
    pub(crate) snake_head_x: i32,
    pub(crate) snake_head_y: i32,
    pub(crate) enemy_head_x: i32,
    pub(crate) enemy_head_y: i32,

    pub(crate) depth: i32,
    pub(crate) score: i32,
    pub(crate) flag: i32,
}

