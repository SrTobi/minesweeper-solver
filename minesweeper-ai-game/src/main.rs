use minesweeper_ai::{GameSetupBuilder, Game, board::BoardVec};

fn main() {
  let mut builder = GameSetupBuilder::new(100, 20);
  builder.add_random_mines(200);

  let mut game = Game::from(builder);
  game.open(BoardVec::new(10, 10));
  println!("{:?}", game);
}
