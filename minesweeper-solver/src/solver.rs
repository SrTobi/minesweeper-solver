use core::fmt;
use std::collections::BinaryHeap;

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

  pub fn deep_suggestion(&self) -> Vec<BoardVec> {
    debug_assert!(self.suggestions().next() == None);
    guess_run(self)
  }

  fn find_guess_positions(&self) -> BinaryHeap<GuessPos> {
    let board = &self.board;
    let mut result = BinaryHeap::new();
    for pos in self.board.positions() {
      if let Explored(explored) = board[pos] {
        if explored.unknowns > 0 && explored.mines > 0 {
          assert!(explored.mines_left > 0);
          let impact = (8 - explored.unknowns) * 1000 / explored.mines_left;
          result.push(GuessPos { impact, pos });
        }
      }
    }

    result
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

#[derive(Clone)]
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

  fn mark_mine(&mut self, pos: BoardVec) -> Result<(), BoardVec> {
    match self.state.board[pos] {
      Unknown => {
        if self.state.mines_left == 0 {
          return Err(pos);
        }
        self.state.mines_left -= 1;
        self.state.board[pos] = Mine;

        for neighbour_pos in pos.neighbours() {
          if let Some(Explored(explored)) = self.state.board.get_mut(neighbour_pos) {
            if explored.mines_left == 0 || explored.unknowns < explored.mines_left {
              return Err(pos);
            }

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

    Ok(())
  }

  fn mark_no_mine(&mut self, pos: BoardVec) -> Result<(), BoardVec> {
    match self.state.board[pos] {
      Unknown => {
        self.state.board[pos] = NoMine;
        for neighbour_pos in pos.neighbours() {
          if let Some(Explored(explored)) = self.state.board.get_mut(neighbour_pos) {
            debug_assert!(explored.unknowns > 0);
            if explored.unknowns <= explored.mines_left {
              return Err(pos);
            }
            explored.unknowns -= 1;
            let explored = *explored;
            self.enqueue(neighbour_pos, explored);
          }
        }
      }
      NoMine | Explored(_) => (),
      Mine => panic!("We deduced that this field must be a mine."),
    }
    Ok(())
  }

  fn enqueue(&mut self, pos: BoardVec, explored: ExploredKnowlede) {
    if explored.conclusion() != Unconclusive {
      self.queue.enqueue(pos);
    }
  }
  pub fn finish(self) -> State {
    self.finish_inner().unwrap()
  }

  fn finish_inner(mut self) -> Result<State, BoardVec> {
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
              self.mark_no_mine(neighbour_pos)?;
            }
          }
        }
        NeighboursAreMines => {
          for neighbour_pos in pos.neighbours() {
            if let Some(Unknown) = self.state.board.get(neighbour_pos) {
              self.mark_mine(neighbour_pos)?;
            }
          }
        }
        _ => (),
      }
    }

    Ok(self.state)
  }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct GuessPos {
  impact: u32,
  pos: BoardVec,
}

impl Ord for GuessPos {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self
      .impact
      .cmp(&other.impact)
      .then_with(|| self.pos.x.cmp(&other.pos.x))
      .then_with(|| self.pos.y.cmp(&other.pos.y))
  }
}

impl PartialOrd for GuessPos {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}

fn guess_run(state: &State) -> Vec<BoardVec> {
  let mut guess_positions = state.find_guess_positions();

  'guess_loop: while let Some(GuessPos { pos, .. }) = guess_positions.pop() {
    //println!("===== {:?} ====", pos);
    let mut succeeded = None;
    let mut result = Vec::new();
    for neighbour_pos in pos.neighbours() {
      if let Some(Unknown) = state.board.get(neighbour_pos) {
        let mut mutator = state.clone().into_mutator();
        mutator.mark_mine(neighbour_pos).unwrap();
        match (mutator.finish_inner(), &succeeded) {
          (Ok(state), Some(succeeded)) if &state != succeeded => {
            //println!("tried:\n{:?}\nHad:\n{:?}", succeeded, state);
            continue 'guess_loop;
          }
          (Ok(state), _) => succeeded = Some(state),
          (Err(_), _) => result.push(neighbour_pos),
        }
      }
    }

    if let Some(state) = succeeded {
      result.extend(state.suggestions());
      result.sort_by(|a, b| a.x.cmp(&b.x).then(a.y.cmp(&a.y)));
      result.dedup();
      return result;
    }
  }

  Vec::new()
}
