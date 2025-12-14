//! file: cell.rs
//! author: Jacob Xie
//! date: 2025/12/14 23:45:17 Sunday
//! brief:

use super::Direction;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
}

impl Cell {
    pub fn offset(self, direction: Direction) -> Self {
        let (dx, dy) = direction.vector();
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}
