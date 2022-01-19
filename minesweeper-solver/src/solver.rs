use core::fmt;

use crate::board::{Board, BoardExplorer, BoardVec};
use crate::{Field, Game};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ExploredKnowlede {
  pub mines: u32,
  pub mines_left: u32,
  pub unknowns: u32,
}

impl ExploredKnowlede {
  pub fn conclusion(&self) -> ExploredKnowledeConclusion {
    if self.unknowns > 0 {
      if self.unknowns == self.mines_left {
        NeighboursAreMines
      } else if self.mines_left == 0 {
        NeighboursAreNotMines
      } else {
        Unconclusive
      }
    } else {
      Unconclusive
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ExploredKnowledeConclusion {
  Unconclusive,
  NeighboursAreMines,
  NeighboursAreNotMines,
}

use ExploredKnowledeConclusion::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FieldKnowledge {
  Unknown,
  Mine,
  NoMine,
  Explored(ExploredKnowlede),
}

use FieldKnowledge::*;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct State {
  board: Board<FieldKnowledge>,
  mines_left: u32,
}

impl State {
  pub fn knowledge_at(&self, pos: BoardVec) -> &FieldKnowledge {
    &self.board[pos]
  }

  pub fn suggestions(&self) -> impl Iterator<Item = BoardVec> + '_ {
    self.board.positions().filter(|&pos| self.board[pos] == NoMine)
  }

  pub fn into_mutator(self) -> StateMutator {
    StateMutator::new(self)
  }
}

impl From<&Game> for State {
  fn from(game: &Game) -> Self {
    let mut mutator = StateMutator::new(State {
      board: Board::new(game.width(), game.height(), Unknown),
      mines_left: game.setup().mines,
    });

    for pos in game.board().positions() {
      if let Some(field) = game.view(pos) {
        mutator.mark_explored(pos, field);
      }
    }

    mutator.finish()
  }
}

impl fmt::Debug for State {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for y in 0..self.board.height {
      for x in 0..self.board.width {
        let pos = BoardVec::new(x as i32, y as i32);
        match self.board[pos] {
          Unknown => write!(f, "░")?,
          Mine => write!(f, "X")?,
          NoMine => write!(f, ".")?,
          Explored(explored) if explored.mines == 0 => write!(f, " ")?,
          Explored(explored) => write!(f, "{}", explored.mines_left)?,
        }
      }
      writeln!(f)?;
    }

    Ok(())
  }
}

pub struct StateMutator {
  state: State,
  queue: BoardExplorer,
}

impl StateMutator {
  pub fn new(state: State) -> Self {
    Self {
      queue: BoardExplorer::from(&state.board),
      state,
    }
  }

  pub fn mark_explored(&mut self, pos: BoardVec, field: Field) {
    match self.state.board[pos] {
      field_knowledge @ (Unknown | NoMine) => {
        if let Field::Empty(mines) = field {
          let mut unknowns = 0;
          let mut mines_left = mines;
          for neighbour_pos in pos.neighbours() {
            match self.state.board.get_mut(neighbour_pos) {
              Some(Explored(explored)) => {
                if field_knowledge == Unknown {
                  debug_assert!(explored.unknowns > 0);
                  explored.unknowns -= 1;
                  let explored = *explored;
                  self.enqueue(neighbour_pos, explored);
                }
              }
              Some(Mine) => {
                debug_assert!(mines_left > 0);
                mines_left -= 1;
              }
              Some(Unknown) => {
                unknowns += 1;
                debug_assert!(unknowns <= 8);
              }
              Some(NoMine) | None => (),
            }
          }
          let knowledge = ExploredKnowlede {
            mines,
            unknowns,
            mines_left,
          };
          self.state.board[pos] = Explored(knowledge);
          self.enqueue(pos, knowledge);
        } else {
          panic!("Cannot explore fields with mines on.")
        }
      }
      Mine => panic!("Cannot mark a field with a mine as explored"),
      Explored(_) => panic!("Already marked as explored"),
    }
  }

  fn mark_mine(&mut self, pos: BoardVec) {
    match self.state.board[pos] {
      Unknown => {
        assert!(self.state.mines_left > 0);
        self.state.mines_left -= 1;
        self.state.board[pos] = Mine;

        for neighbour_pos in pos.neighbours() {
          if let Some(Explored(explored)) = self.state.board.get_mut(neighbour_pos) {
            debug_assert!(explored.mines_left > 0);
            debug_assert!(explored.unknowns > 0);
            explored.mines_left -= 1;
            explored.unknowns -= 1;
            let explored = *explored;
            self.enqueue(neighbour_pos, explored);
          }
        }
      }
      Mine => (),
      Explored(_) | NoMine => panic!("We deduced that this field cannot be a mine."),
    }
  }

  fn mark_no_mine(&mut self, pos: BoardVec) {
    match self.state.board[pos] {
      Unknown => {
        self.state.board[pos] = NoMine;
        for neighbour_pos in pos.neighbours() {
          if let Some(Explored(explored)) = self.state.board.get_mut(neighbour_pos) {
            debug_assert!(explored.unknowns > 0);
            explored.unknowns -= 1;
            let explored = *explored;
            self.enqueue(neighbour_pos, explored);
          }
        }
      }
      NoMine | Explored(_) => (),
      Mine => panic!("We deduced that this field must be a mine."),
    }
  }

  fn enqueue(&mut self, pos: BoardVec, explored: ExploredKnowlede) {
    if explored.conclusion() != Unconclusive {
      self.queue.enqueue(pos);
    }
  }

  pub fn finish(mut self) -> State {
    self.queue.set_allow_multiple_enqueue(true);
    while let Some(pos) = self.queue.pop() {
      let explored = if let Explored(explored) = &self.state.board[pos] {
        explored
      } else {
        panic!("Only explored fields can be of interest.")
      };
      match explored.conclusion() {
        NeighboursAreNotMines => {
          for neighbour_pos in pos.neighbours() {
            if let Some(Unknown) = self.state.board.get(neighbour_pos) {
              self.mark_no_mine(neighbour_pos);
            }
          }
        }
        NeighboursAreMines => {
          for neighbour_pos in pos.neighbours() {
            if let Some(Unknown) = self.state.board.get(neighbour_pos) {
              self.mark_mine(neighbour_pos);
            }
          }
        }
        _ => (),
      }
    }

    self.state
  }
}
