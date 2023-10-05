use std::fmt;

use super::commons::*;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Fruit {
    Grape,
    Banana,
    Strawberry,
}

impl Fruit {
    pub fn points(&self) -> usize {
        match &self {
            Fruit::Grape => 16,
            Fruit::Banana => 32,
            Fruit::Strawberry => 64,
        }
    }
}

#[derive(Debug)]
pub struct FruitPos {
    pub fruit: Fruit,
    pub pos: Position,
}

impl fmt::Display for Fruit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Fruit::Grape => write!(f, "G"),
            Fruit::Banana => write!(f, "B"),
            Fruit::Strawberry => write!(f, "F"),
        }
    }
}
