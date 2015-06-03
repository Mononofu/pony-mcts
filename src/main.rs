extern crate rand;
use rand::Rng;
extern crate time;

mod go;

fn main() {
  let mut game = go::GoGame::new(19);
  let mut rng = rand::thread_rng();
  let mut color_to_play = go::Stone::Black;
  let mut possible_moves = game.possible_moves(color_to_play);
  let mut num_consecutive_passes = 0;
  let mut num_moves = 0;
  let start = time::PreciseTime::now();

  while num_consecutive_passes < 2 {
    num_moves += 1;
    match rng.choose(&possible_moves) {
      Some(m) => {
        num_consecutive_passes = 0;
        game.play(color_to_play, *m, false);
        // println!("{}", game);
      }
      None => {
        num_consecutive_passes += 1;
        // println!("{:?} passes", color_to_play);
      }
    }
    color_to_play = color_to_play.opponent();
    possible_moves = game.possible_moves(color_to_play);
  //   std::thread::sleep_ms(500);
  }
  println!("{} moves in {}", num_moves, start.to(time::PreciseTime::now()));
}
