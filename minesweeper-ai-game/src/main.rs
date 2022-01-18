use minesweeper_ai::{GameSetupBuilder, Game};

fn main() {
  let mut builder = GameSetupBuilder::new(15, 15);
  builder.add_random_mines(20);

  let game = Game::from(builder);
  println!("{:?}", game);
}
