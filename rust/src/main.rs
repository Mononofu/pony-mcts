extern crate rand;
use rand::SeedableRng;
extern crate time;

mod go;

fn main() {
  let num_trials = 11;
  let num_rollouts = 10000;
  let mut durations = (0..num_trials).map(|_| benchmark_run(num_rollouts)).collect::<Vec<_>>();
  durations.sort();
  let mean = durations.iter().fold(time::Duration::zero(), |acc, &d| acc + d) / num_trials;
  let median = durations[(num_trials / 2) as usize];
  let min = durations[0];
  let max = durations[(num_trials - 1) as usize];
  let mut stddev = 0.0;
  for d in durations {
    let diff = (d.num_nanoseconds().unwrap() - mean.num_nanoseconds().unwrap()) as f64;
    stddev += diff * diff;
  }
  let stddev_dur = time::Duration::nanoseconds(stddev.sqrt() as i64);
  println!("|{}---{}---{}|, mean {} +- {}", min, median, max, mean, stddev_dur);
}

fn benchmark_run(num_playouts: i32) -> time::Duration {
  let start = time::PreciseTime::now();
  let mut rng = rand::StdRng::from_seed(&[42]);
  let mut num_moves = 0u64;
  let mut double_total_score = 0i64;
  let mut num_black_wins = 0u64;
  for _ in 0 .. num_playouts {
    let (n, s) = play(&mut rng);
    num_moves += n as u64;
    double_total_score += s as i64;
    if s > 0 {
      num_black_wins += 1;
    }
  }
  let total = start.to(time::PreciseTime::now());
  println!("{} playouts in {}, {:.2} kpps", num_playouts, total,
      num_playouts as f64 / total.num_milliseconds() as f64);
  println!("{} moves per playout, mean score {:.2}, winrate {:.2} %",
      num_moves as f64 / num_playouts as f64,
      double_total_score as f64 / num_playouts as f64 / 2f64,
      100f64 * num_black_wins as f64 / num_playouts as f64);
  return total;
}

fn play(rng: &mut rand::StdRng) -> (u32, i16) {
  // Use doubled score so we can score 0.5 komi in integer.
  let double_komi = 15;
  let mut game = go::GoGame::new(19);
  let mut color_to_play = go::Stone::White;
  let mut num_consecutive_passes = 0;
  let mut num_moves = 0;

  'outer: while num_consecutive_passes < 2 {
    color_to_play = color_to_play.opponent();
    num_moves += 1;
    let v = game.random_move(color_to_play, rng);
    if v == go::PASS {
      num_consecutive_passes += 1;
    } else {
      game.play(color_to_play, v);
      num_consecutive_passes = 0;
    }
    if num_moves > 700 {
      println!("{}", game);
    }
    if num_moves > 710 {
      println!("suspicious game with > 700 moves");
      break;
    }
  }
  return (num_moves, game.chinese_score() * 2 - double_komi);
}
