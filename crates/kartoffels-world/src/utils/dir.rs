use glam::{ivec2, IVec2};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use serde::{Deserialize, Serialize};

// TODO rename to north/east/west/south
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dir {
    #[serde(rename = "^")]
    Up,

    #[serde(rename = ">")]
    Right,

    #[serde(rename = "v")]
    Down,

    #[serde(rename = "<")]
    Left,
}

impl Dir {
    pub fn all() -> impl Iterator<Item = Self> {
        [Dir::Up, Dir::Right, Dir::Down, Dir::Left].into_iter()
    }

    #[must_use]
    pub fn turned_left(self) -> Self {
        match self {
            Dir::Up => Dir::Left,
            Dir::Right => Dir::Up,
            Dir::Down => Dir::Right,
            Dir::Left => Dir::Down,
        }
    }

    #[must_use]
    pub fn turned_right(self) -> Self {
        match self {
            Dir::Up => Dir::Right,
            Dir::Right => Dir::Down,
            Dir::Down => Dir::Left,
            Dir::Left => Dir::Up,
        }
    }

    #[must_use]
    pub fn as_vec(&self) -> IVec2 {
        match self {
            Dir::Up => ivec2(0, -1),
            Dir::Right => ivec2(1, 0),
            Dir::Down => ivec2(0, 1),
            Dir::Left => ivec2(-1, 0),
        }
    }
}

impl Distribution<Dir> for Standard {
    fn sample<R>(&self, rng: &mut R) -> Dir
    where
        R: Rng + ?Sized,
    {
        match rng.gen_range(0..4) {
            0 => Dir::Up,
            1 => Dir::Right,
            2 => Dir::Down,
            _ => Dir::Left,
        }
    }
}

impl From<u8> for Dir {
    fn from(value: u8) -> Self {
        match value {
            0 => Dir::Up,
            1 => Dir::Right,
            2 => Dir::Down,
            _ => Dir::Left,
        }
    }
}

impl From<Dir> for u8 {
    fn from(value: Dir) -> Self {
        match value {
            Dir::Up => 0,
            Dir::Right => 1,
            Dir::Down => 2,
            Dir::Left => 3,
        }
    }
}
