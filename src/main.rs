extern crate rand;
use rand::Rng;
use rand::SeedableRng;
extern crate time;

mod go;
mod bench;

fn main() {
  for _ in 0 .. 1 {
    play();
  }
}

fn play() {
  let mut game = go::GoGame::new(19);
  let mut rng = rand::StdRng::from_seed(&[42]);
  let mut color_to_play = go::Stone::Black;
  let mut empty_vertices = game.empty_vertices();
  let mut num_consecutive_passes = 0;
  let mut num_moves = 0;
  let start = time::PreciseTime::now();

  while num_consecutive_passes < 2 {
    num_moves += 1;
    rng.shuffle(&mut empty_vertices);
    num_consecutive_passes += 1;
    for v in empty_vertices.iter() {
      if game.can_play(color_to_play, *v) {
        num_consecutive_passes = 0;
        game.play(color_to_play, *v, false);
        break;
      }
    }
    color_to_play = color_to_play.opponent();
    empty_vertices = game.empty_vertices()
    // std::thread::sleep_ms(100);
  }
  println!("{} moves in {}", num_moves, start.to(time::PreciseTime::now()));
  // println!("{}", game);
}
