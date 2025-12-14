//! file: status.rs
//! author: Jacob Xie
//! date: 2025/12/14 23:44:54 Sunday
//! brief:

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    Ready,
    Running,
    Paused,
    GameOver,
}
