use minesweeper_solver::board::BoardVec;
use minesweeper_solver::solver::State;
use minesweeper_solver::{Game, GameSetupBuilder};

fn make_game() -> Game {
  let start = BoardVec::new(100, 20);
  loop {
    let mut builder = GameSetupBuilder::new(200, 40);
    builder.protect_all(start.with_neighbours());
    builder.add_random_mines(1400);
  
    let mut game = Game::from(builder);
    game.open(start);
    if game.clone().is_solvable() {
      return game;
    }
  }
}

fn main() {
  //println!("{:?}", game);
  let mut game = make_game();
  let mut state = State::from(&game);

  loop {
    let mut suggestions = state.suggestions().collect::<Vec<_>>();

    println!("{:?}", state);
    println!("{:?}", game);

    if game.is_win() {
      println!("Win!");
      return;
    }

    if suggestions.is_empty() {
      println!("No suggestions.. try to guess...");
      suggestions = state.deep_suggestion();
      if suggestions.is_empty() {
        println!("Not solvable!");
        return;
      }
      println!("Guessed {:?}", suggestions);
    }

    let mut mutator = state.into_mutator();
    for suggestion in suggestions {
      for opened in game.open(suggestion).unwrap() {
        mutator.mark_explored(opened, game.view(opened).unwrap())
      }
    }

    state = mutator.finish();
  }
}
