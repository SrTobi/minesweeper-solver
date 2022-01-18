use std::ops::{Add, Index, IndexMut, Neg, Sub};

pub static NORTH: BoardVec = BoardVec::new(0, -1);
pub static NORTH_EAST: BoardVec = BoardVec::new(1, -1);
pub static EAST: BoardVec = BoardVec::new(1, 0);
pub static SOUTH_EAST: BoardVec = BoardVec::new(1, 1);
pub static SOUTH: BoardVec = BoardVec::new(0, 1);
pub static SOUTH_WEST: BoardVec = BoardVec::new(-1, 1);
pub static WEST: BoardVec = BoardVec::new(-1, 0);
pub static NORTH_WEST: BoardVec = BoardVec::new(-1, -1);
pub static CENTER: BoardVec = BoardVec::new(0, 0);

pub static DIRECTIONS: [BoardVec; 8] = [NORTH_WEST, NORTH, NORTH_EAST, EAST, SOUTH_EAST, SOUTH, SOUTH_WEST, WEST];
pub static CENTER_AND_DIRECTIONS: [BoardVec; 9] = [
  NORTH_WEST, NORTH, NORTH_EAST, WEST, CENTER, EAST, SOUTH_WEST, SOUTH, SOUTH_EAST,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct BoardVec {
  pub x: i32,
  pub y: i32,
}

impl BoardVec {
  pub const fn new(x: i32, y: i32) -> BoardVec {
    BoardVec { x, y }
  }
}

impl Add<BoardVec> for BoardVec {
  type Output = BoardVec;

  fn add(self, rhs: BoardVec) -> Self::Output {
    BoardVec::new(self.x + rhs.x, self.y + rhs.y)
  }
}

impl Sub<BoardVec> for BoardVec {
  type Output = BoardVec;

  fn sub(self, rhs: BoardVec) -> Self::Output {
    BoardVec::new(self.x - rhs.x, self.y - rhs.y)
  }
}

impl Neg for BoardVec {
  type Output = BoardVec;

  fn neg(self) -> Self::Output {
    BoardVec::new(-self.x, -self.y)
  }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Board<T> {
  pub width: u32,
  pub height: u32,
  fields: Vec<T>,
}

impl<T> Board<T> {
  pub fn new(width: u32, height: u32, default: T) -> Self
  where
    T: Clone,
  {
    Self {
      width,
      height,
      fields: vec![default; (width * height) as usize],
    }
  }

  fn pos_to_index(&self, pos: BoardVec) -> usize {
    usize::try_from(pos.x).unwrap() + (usize::try_from(pos.y).unwrap() * (self.height as usize))
  }

  pub fn get(&self, pos: BoardVec) -> Option<&T> {
    self.fields.get(self.pos_to_index(pos))
  }

  pub fn get_mut(&mut self, pos: BoardVec) -> Option<&mut T> {
    let index = self.pos_to_index(pos);
    self.fields.get_mut(index)
  }

  pub fn get_around(&mut self, pos: BoardVec) -> impl Iterator<Item = &T> {
    self.positions_around(pos).flat_map(|pos| self.get(pos))
  }

  pub fn positions(&self) -> BoardPositionIterator {
    BoardPositionIterator::new(BoardVec::new(0, 0), self.width, self.height)
  }

  pub fn positions_around(&self, pos: BoardVec) -> impl Iterator<Item = BoardVec> {
    DIRECTIONS.iter().map(move |&dir| dir + pos)
  }
}

impl<T> Index<BoardVec> for Board<T> {
  type Output = T;

  fn index(&self, index: BoardVec) -> &Self::Output {
    self.get(index).unwrap()
  }
}

impl<T> IndexMut<BoardVec> for Board<T> {
  fn index_mut(&mut self, index: BoardVec) -> &mut T {
    self.get_mut(index).unwrap()
  }
}

pub struct BoardPositionIterator {
  next_pos: BoardVec,
  x_start: i32,
  x_end: i32,
  y_end: i32,
}

impl BoardPositionIterator {
  pub fn new(pos: BoardVec, width: u32, height: u32) -> Self {
    let y_end = pos.y + height as i32;
    Self {
      next_pos: if width == 0 { BoardVec::new(0, y_end) } else { pos },
      x_start: pos.x,
      x_end: pos.x + width as i32,
      y_end,
    }
  }
}

impl Iterator for BoardPositionIterator {
  type Item = BoardVec;

  fn next(&mut self) -> Option<Self::Item> {
    let pos = &mut self.next_pos;
    if pos.x >= self.y_end {
      None
    } else {
      let result = *pos;
      pos.x += 1;
      if pos.x >= self.x_end {
        pos.x = self.x_start;
        pos.y += 1;
      }
      Some(result)
    }
  }
}
