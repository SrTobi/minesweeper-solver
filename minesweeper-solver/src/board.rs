use core::fmt;
use std::collections::VecDeque;
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

pub static DIRECTIONS: [BoardVec; 8] = [NORTH_WEST, NORTH, NORTH_EAST, WEST, EAST, SOUTH_WEST, SOUTH, SOUTH_EAST];
pub static CENTER_AND_DIRECTIONS: [BoardVec; 9] = [
  NORTH_WEST, NORTH, NORTH_EAST, WEST, CENTER, EAST, SOUTH_WEST, SOUTH, SOUTH_EAST,
];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct BoardVec {
  pub x: i32,
  pub y: i32,
}

impl BoardVec {
  pub const fn new(x: i32, y: i32) -> BoardVec {
    BoardVec { x, y }
  }

  pub fn with_neighbours(self) -> impl Iterator<Item = BoardVec> {
    CENTER_AND_DIRECTIONS.iter().map(move |&dir| dir + self)
  }

  pub fn neighbours(self) -> impl Iterator<Item = BoardVec> {
    DIRECTIONS.iter().map(move |&dir| dir + self)
  }
}

impl fmt::Debug for BoardVec {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "({}, {})", self.x, self.y)
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

  fn pos_to_index(&self, pos: BoardVec) -> Option<usize> {
    match (usize::try_from(pos.x), usize::try_from(pos.y)) {
      (Ok(x), Ok(y)) if x < self.width as usize && y < self.height as usize => Some(x + y * (self.width as usize)),
      _ => None,
    }
  }

  pub fn get(&self, pos: BoardVec) -> Option<&T> {
    self.pos_to_index(pos).and_then(|i| self.fields.get(i))
  }

  pub fn get_mut(&mut self, pos: BoardVec) -> Option<&mut T> {
    self.pos_to_index(pos).and_then(|i| self.fields.get_mut(i))
  }

  pub fn get_around(&self, pos: BoardVec) -> impl Iterator<Item = &T> {
    pos.neighbours().flat_map(|pos| self.get(pos))
  }

  pub fn positions(&self) -> BoardPositionIterator {
    BoardPositionIterator::new(BoardVec::new(0, 0), self.width, self.height)
  }
  pub fn enumerate(&self) -> impl Iterator<Item = (BoardVec, &T)> {
    self.positions().zip(self.fields.iter())
  }

  pub fn enumerate_mut(&mut self) -> impl Iterator<Item = (BoardVec, &mut T)> {
    self.positions().zip(self.fields.iter_mut())
  }

  pub fn iter(&self) -> impl Iterator<Item = &T> {
    self.fields.iter()
  }
}

impl<T> Index<BoardVec> for Board<T> {
  type Output = T;

  fn index(&self, index: BoardVec) -> &Self::Output {
    self.get(index).unwrap_or_else(|| {
      panic!(
        "Cannot access position {:?} on board with size {}x{}",
        index, self.width, self.height
      )
    })
  }
}

impl<T> IndexMut<BoardVec> for Board<T> {
  fn index_mut(&mut self, index: BoardVec) -> &mut T {
    let (width, height) = (self.width, self.height);
    self.get_mut(index).unwrap_or_else(|| {
      panic!(
        "Cannot mut-access position {:?} on board with size {}x{}",
        index, width, height
      )
    })
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
    if pos.y >= self.y_end {
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

#[derive(Clone)]
pub struct BoardExplorer {
  queue: VecDeque<BoardVec>,
  visited: Board<bool>,
  allow_multi: bool,
}

impl BoardExplorer {
  pub fn set_allow_multiple_enqueue(&mut self, b: bool) {
    self.allow_multi = b;
  }

  pub fn enqueue(&mut self, pos: BoardVec) -> bool {
    if let Some(field) = self.visited.get_mut(pos) {
      if !*field {
        *field = true;
        self.queue.push_back(pos);
        return true;
      }
    }
    false
  }

  pub fn enqueue_all(&mut self, all: impl IntoIterator<Item = BoardVec>) {
    for pos in all {
      self.enqueue(pos);
    }
  }

  pub fn pop(&mut self) -> Option<BoardVec> {
    let result = self.queue.pop_front();
    if self.allow_multi {
      if let Some(pos) = result {
        debug_assert!(self.visited[pos]);
        self.visited[pos] = false;
      }
    }
    result
  }
}

impl<T> From<&Board<T>> for BoardExplorer {
  fn from(board: &Board<T>) -> Self {
    Self {
      queue: VecDeque::new(),
      visited: Board::new(board.width, board.height, false),
      allow_multi: false,
    }
  }
}
