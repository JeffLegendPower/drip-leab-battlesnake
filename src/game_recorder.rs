pub struct RoundInfo {
    pub(crate) snake_bfs: i32,
    pub(crate) enemy_bfs: i32,
    pub(crate) snake_length: i32,
    pub(crate) enemy_length: i32,
    pub(crate) health: i32
}

pub struct GameRecorder {
    pub(crate) rounds: Vec<RoundInfo>,
    pub(crate) ending: f32
}