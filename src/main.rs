extern crate rand;
use rand::Rng;
use rand::SeedableRng;
extern crate time;

mod go;
mod bench;

fn main() {
  let num_playouts = 100000;
  let start = time::PreciseTime::now();
  let mut rng = rand::StdRng::from_seed(&[42]);
  for _ in 0 .. num_playouts {
    play(&mut rng);
  }
  let total = start.to(time::PreciseTime::now());
  println!("{} playouts in {}, {} per playout", num_playouts, total,
      total / num_playouts);
}

fn play(rng: &mut rand::StdRng) {
  let mut game = go::GoGame::new(19);
  let mut color_to_play = go::Stone::Black;
  let mut empty_vertices = game.empty_vertices();
  let mut num_consecutive_passes = 0;
  let mut num_moves = 0;

  'outer: while num_consecutive_passes < 2 {
    num_moves += 1;
    num_consecutive_passes += 1;
    'inner: for i in 0 .. 10 {
      let v = rng.choose(&empty_vertices).unwrap();
      if game.can_play(color_to_play, *v) {
        num_consecutive_passes = 0;
        game.play(color_to_play, *v, false);
        break 'outer;
      }
    }
    rng.shuffle(&mut empty_vertices);
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
  // println!("{} moves", num_moves);
  // game.report();
  // println!("{}", game);
}
