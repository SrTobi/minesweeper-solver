use core::fmt;
use std::borrow::Borrow;

use board::{Board, BoardVec};
use rand::prelude::SliceRandom;
use rand::RngCore;
use solver::State;

use crate::board::BoardExplorer;

pub mod board;
pub mod solver;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Field {
  Mine,
  Empty(u32),
}

impl Field {
  pub fn is_mine(self) -> bool {
    matches!(self, Field::Mine)
  }

  pub fn is_blank(self) -> bool {
    matches!(self, Field::Empty(0))
  }

  fn notify_mine(field: &mut Field) {
    if let Field::Empty(mines) = field {
      *mines += 1;
      //assert!(*mines <= 8);
    }
  }
}

impl fmt::Display for Field {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Field::Mine => write!(f, "X"),
      Field::Empty(0) => write!(f, " "),
      Field::Empty(mines) => write!(f, "{}", mines),
    }
  }
}

pub type GameBoard = Board<Field>;
pub type ViewBoard = Board<bool>;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct GameSetup {
  board: GameBoard,
  mines: u32,
}

impl GameSetup {
  pub fn new(bombs: &Board<bool>) -> Self {
    let mut board = GameBoard::new(bombs.width, bombs.height, Field::Empty(0));
    let mut mines = 0;
    for (pos, &is_mine) in bombs.enumerate() {
      if is_mine {
        mines += 1;
        board[pos] = Field::Mine;
        for neighbour_pos in pos.neighbours() {
          if let Some(neighbour) = board.get_mut(neighbour_pos) {
            Field::notify_mine(neighbour);
          }
        }
      }
    }

    GameSetup { board, mines }
  }

  pub fn width(&self) -> u32 {
    self.board.width
  }

  pub fn height(&self) -> u32 {
    self.board.height
  }
}

impl<B: Borrow<GameSetupBuilder>> From<B> for GameSetup {
  fn from(builder: B) -> Self {
    let builder: &GameSetupBuilder = builder.borrow();
    Self::new(&builder.mines)
  }
}

impl fmt::Debug for GameSetup {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for y in 0..self.height() {
      for x in 0..self.width() {
        let pos = BoardVec::new(x as i32, y as i32);
        write!(f, "{}", self.board[pos])?;
      }
      writeln!(f)?;
    }

    Ok(())
  }
}

pub struct GameSetupBuilder {
  mines: Board<bool>,
  protected: Board<bool>,
  rng: Box<dyn RngCore>,
}

impl GameSetupBuilder {
  pub fn new(width: u32, height: u32) -> Self {
    Self {
      mines: Board::new(width, height, false),
      protected: Board::new(width, height, false),
      rng: Box::new(rand::thread_rng()),
    }
  }

  pub fn has_mine(&self, pos: BoardVec) -> bool {
    self.mines[pos]
  }

  pub fn set_mine(&mut self, pos: BoardVec) {
    assert!(!self.is_protected(pos));
    self.mines[pos] = true;
  }

  pub fn is_protected(&self, pos: BoardVec) -> bool {
    self.protected[pos]
  }

  pub fn protect(&mut self, pos: BoardVec) {
    if self.mines.get(pos).is_some() {
      self.mines[pos] = false;
      self.protected[pos] = true;
    }
  }

  pub fn protect_all(&mut self, all: impl IntoIterator<Item = BoardVec>) {
    for pos in all {
      self.protect(pos);
    }
  }

  pub fn add_random_mines(&mut self, mut mines: u32) -> bool {
    let mut possible_positions: Vec<_> = self.mines.positions().collect();
    possible_positions.shuffle(&mut self.rng);

    while let Some(pos) = possible_positions.pop() {
      if mines == 0 {
        return true;
      }

      if self.is_protected(pos) || self.has_mine(pos) {
        continue;
      }

      self.set_mine(pos);
      mines -= 1;
    }

    false
  }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Game {
  setup: GameSetup,
  view: ViewBoard,
  hidden_fields: u32,
}

impl Game {
  pub fn setup(&self) -> &GameSetup {
    &self.setup
  }

  pub fn is_win(&self) -> bool {
    self.hidden_fields == self.setup.mines
  }

  pub fn board(&self) -> &GameBoard {
    &self.setup.board
  }

  pub fn width(&self) -> u32 {
    self.board().width
  }

  pub fn height(&self) -> u32 {
    self.board().height
  }

  pub fn is_visible(&self, pos: BoardVec) -> bool {
    self.view[pos]
  }

  pub fn view(&self, pos: BoardVec) -> Option<Field> {
    if self.is_visible(pos) {
      self.board().get(pos).copied()
    } else {
      None
    }
  }

  pub fn open(&mut self, pos: BoardVec) -> Option<Vec<BoardVec>> {
    //assert!(!self.is_visible(pos));
    if self.board()[pos].is_mine() {
      return None;
    }

    let mut explorer = BoardExplorer::from(self.board());
    explorer.enqueue(pos);

    let mut opened = Vec::new();
    while let Some(pos) = explorer.pop() {
      if !self.is_visible(pos) {
        self.view[pos] = true;
        self.hidden_fields -= 1;
        debug_assert!(self.hidden_fields >= self.setup.mines);
        opened.push(pos);
        if self.board()[pos].is_blank() {
          explorer.enqueue_all(pos.neighbours());
        }
      }
    }

    Some(opened)
  }

  // todo: better tip 
  pub fn tipp(&self) -> Vec<BoardVec> {
    let state = State::from(self);

    let mut suggestions: Vec<BoardVec> = state.suggestions().collect();
    if suggestions.is_empty() {
      suggestions = state.deep_suggestion();
    }
    suggestions
  }

  pub fn is_solvable(mut self) -> bool {
    let mut state = State::from(&self);
    loop {
      if self.is_win() {
        return true;
      }

      let mut suggestions = state.suggestions().collect::<Vec<_>>();
      if suggestions.is_empty() {
        suggestions = state.deep_suggestion();
        if suggestions.is_empty() {
          return false;
        }
      }

      let mut mutator = state.into_mutator();
      for suggestion in suggestions {
        for opened in self.open(suggestion).unwrap() {
          mutator.mark_explored(opened, self.view(opened).unwrap())
        }
      }

      state = mutator.finish();
    }
  }
}

impl From<GameSetup> for Game {
  fn from(setup: GameSetup) -> Self {
    Self {
      view: ViewBoard::new(setup.width(), setup.height(), false),
      hidden_fields: setup.width() * setup.height(),
      setup,
    }
  }
}

impl<B: Borrow<GameSetupBuilder>> From<B> for Game {
  fn from(setup: B) -> Self {
    Self::from(GameSetup::from(setup))
  }
}

impl fmt::Debug for Game {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for y in 0..self.height() {
      for x in 0..self.width() {
        let pos = BoardVec::new(x as i32, y as i32);
        if self.is_visible(pos) {
          write!(f, "{}", self.board()[pos])?;
        } else {
          write!(f, "░")?;
        }
      }
      writeln!(f)?;
    }

    Ok(())
  }
}

/*
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum FieldView {
  Open,
  Hidden,
  Flagged,
}

impl FieldView {
  pub fn is_open(self) -> bool {
    self == FieldView::Open
  }

  pub fn is_hidden(self) -> bool {
    !self.is_open()
  }

  pub fn is_flagged(self) -> bool {
    self == FieldView::Flagged
  }
}*/
