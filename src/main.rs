extern crate rand;
use rand::Rng;
use rand::SeedableRng;
extern crate time;

mod go;

fn main() {
  let num_playouts = 5000;
  let start = time::PreciseTime::now();
  let mut rng = rand::StdRng::from_seed(&[42]);
  let mut num_moves = 0u64;
  for _ in 0 .. num_playouts {
    let n = play(&mut rng);
    num_moves += n as u64;
  }
  let total = start.to(time::PreciseTime::now());
  println!("{} playouts in {}, {} per playout", num_playouts, total,
      total / num_playouts);
  println!("{} moves per playout", num_moves as f64 / num_playouts as f64);
}

fn play(rng: &mut rand::StdRng) -> u32 {
  let mut game = go::GoGame::new(19);
  let mut color_to_play = go::Stone::White;
  let mut empty_vertices;
  let mut num_consecutive_passes = 0;
  let mut num_moves = 0;

  'outer: while num_consecutive_passes < 2 {
    color_to_play = color_to_play.opponent();
    empty_vertices = game.empty_vertices();
    num_moves += 1;
    num_consecutive_passes += 1;
    'inner: for _ in 0 .. 10 {
      let v = rng.choose(&empty_vertices).unwrap();
      if game.play(color_to_play, *v) {
        num_consecutive_passes = 0;
        continue 'outer;
      }
    }
    rng.shuffle(&mut empty_vertices);
    for v in empty_vertices.iter() {
      if game.play(color_to_play, *v) {
        num_consecutive_passes = 0;
        break;
      }
    }
  }
  return num_moves;
}
