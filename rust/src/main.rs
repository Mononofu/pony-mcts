#[macro_use]
extern crate log;
extern crate rand;
use rand::SeedableRng;
extern crate time;
use std::io;
use std::io::prelude::*;

mod go;
mod mcts;
mod gtp;

use log::{LogRecord, LogLevel, LogLevelFilter, LogMetadata};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            writeln!(&mut std::io::stderr(), "{} {}:{} - {}", record.level(),
              record.location().file(), record.location().line(),
              record.args());
        }
    }
}

#[allow(dead_code)]
fn main() {
  log::set_logger(|max_log_level| {
        max_log_level.set(LogLevelFilter::Info);
        Box::new(SimpleLogger)
    }).unwrap();

  // let mut rng = rand::StdRng::from_seed(&[time::precise_time_ns() as usize]);
  let rng = rand::StdRng::from_seed(&[42]);

  let mut engine = gtp::Engine::new(rng);
  let stdin = io::stdin();
  for line in stdin.lock().lines() {
    println!("{}", engine.execute(line.unwrap()));
    println!("");
    if !engine.running {
      return;
    }
  }

  benchmark(run_rollouts, 1000, 1);
}

fn benchmark(f: fn(u64), n: u64, repetitions: u64) {
  let mut durations = (0..repetitions).map(|_| {
    let start = time::PreciseTime::now();
    f(n);
    let total = start.to(time::PreciseTime::now());
    println!("{} playouts in {}, {:.2} kpps", n, total,
      n as f64 / total.num_milliseconds() as f64);
    total
  }).collect::<Vec<_>>();
  durations.sort();

  let mean = durations.iter().fold(time::Duration::zero(), |acc, &d| acc + d) / repetitions as i32;
  let median = durations[(repetitions / 2) as usize];
  let min = durations[0];
  let max = durations[(repetitions - 1) as usize];

  let mut stddev = 0.0;
  for d in durations {
    let diff = (d.num_nanoseconds().unwrap() - mean.num_nanoseconds().unwrap()) as f64;
    stddev += diff * diff;
  }
  let stddev_dur = time::Duration::nanoseconds(stddev.sqrt() as i64);

  println!("|{}---{}---{}|, mean {} +- {}", min, median, max, mean, stddev_dur);
}

fn run_rollouts(num_rollouts: u64) {
  let mut rng = rand::StdRng::from_seed(&[42]);
  let mut num_moves = 0u64;
  let mut double_total_score = 0i64;
  let mut game = go::GoGame::new(19);
  let mut num_black_wins = 0u64;
  for _ in 0 .. num_rollouts {
    let (n, s) = play(&mut game, &mut rng);
    num_moves += n as u64;
    double_total_score += s as i64;
    if s > 0 {
      num_black_wins += 1;
    }
  }
  println!("{} moves per playout, mean score {:.2}, winrate {:.2} %",
      num_moves as f64 / num_rollouts as f64,
      double_total_score as f64 / num_rollouts as f64 / 2f64,
      100f64 * num_black_wins as f64 / num_rollouts as f64);
}

fn play(game: &mut go::GoGame, rng: &mut rand::StdRng) -> (u32, i16) {
  // Use doubled score so we can score 0.5 komi in integer.
  let double_komi = 15;
  let mut color_to_play = go::stone::WHITE;
  let mut num_consecutive_passes = 0;
  let mut num_moves = 0;
  game.reset();

  while num_consecutive_passes < 2 {
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
