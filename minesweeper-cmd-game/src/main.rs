use minesweeper_solver::board::BoardVec;
use minesweeper_solver::solver::State;
use minesweeper_solver::{Game, GameSetupBuilder};

fn main() {
  let mut builder = GameSetupBuilder::new(200, 40);
  builder.protect_all(BoardVec::new(1, 1).with_neighbours());
  builder.add_random_mines(1400);

  let mut game = Game::from(builder);
  game.open(BoardVec::new(1, 1));
  //println!("{:?}", game);

  let mut state = State::from(&game);

  loop {
    let suggestions = state.suggestions().collect::<Vec<_>>();

    println!("{:?}", state);

    if suggestions.is_empty() {
      println!("{:?}", game);
      return;
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
