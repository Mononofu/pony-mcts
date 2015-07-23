extern crate log;
extern crate rand;

use go::Vertex;
use go::PASS;
use go::GoGame;
use go::Stone;
use go::stone;

const EXPANSION_THRESHOLD: u32 = 4;
const UCT_C: f64 = 1.4;

pub struct Node {
  player: Stone,
  pub vertex: Vertex,
  pub children: Vec<Node>,

  num_plays: u32,
  num_wins: i32,
}

pub struct Controller {
  pub root: Node,
}

fn black_wins(game: &mut GoGame, last_move: Stone, rng: &mut rand::StdRng) -> bool {
  let double_komi = 13;
  let mut color_to_play = last_move;
  let mut num_consecutive_passes = 0;
  let mut num_moves = 0;

  while num_consecutive_passes < 2 {
    color_to_play = color_to_play.opponent();
    num_moves += 1;
    let v = game.random_move(color_to_play, rng);
    if v == PASS {
      num_consecutive_passes += 1;
    } else {
      game.play(color_to_play, v);
      num_consecutive_passes = 0;
    }
    if num_moves > 700 {
      warn!("too many moves!");
      return false;
    }
  }
  return game.chinese_score() * 2 - double_komi > 0;
}

impl Controller {
  pub fn new() -> Controller {
    Controller {
      root: Node::new(stone::WHITE, PASS),
    }
  }

  pub fn gen_move(&mut self, game: &GoGame, num_rollouts: u32, rng: &mut rand::StdRng) -> Vertex {
    let mut rollout_game = game.clone();
    if rollout_game.possible_moves(game.to_play).is_empty() {
      return PASS;
    }

    self.root = Node::new(game.to_play.opponent(), PASS);
    for i in 1 .. num_rollouts + 1 {
      rollout_game.reset();
      for v in game.history.iter() {
        let to_play = rollout_game.to_play;
        rollout_game.play(to_play, *v);
      }
      self.root.run_rollout(i, &mut rollout_game, rng);
    }
    if self.root.children.is_empty() {
      return PASS;
    }
    self.root.best_child(num_rollouts, 0f64).vertex
  }
}

impl Node {
  fn new(player: Stone, vertex: Vertex) -> Node {
    Node {
      player: player,
      vertex: vertex,
      children: vec![],

      num_plays: 2,
      num_wins: 1,
    }
  }

  // Returns whether black wins to update the win rate in parent nodes.
  fn run_rollout(&mut self, num_sims: u32, game: &mut GoGame, rng: &mut rand::StdRng) -> bool {
    game.play(self.player, self.vertex);
    self.num_plays += 1;

    let black_wins = if self.children.is_empty() {
      if self.num_plays > EXPANSION_THRESHOLD {
        let opponent = self.player.opponent();
        for v in game.possible_moves(opponent) {
          self.children.push(Node::new(opponent, v));
        }
      }

      black_wins(game, self.player, rng)
    } else {
      self.best_child(num_sims, UCT_C).run_rollout(num_sims, game, rng)
    };

    if black_wins && self.player == stone::BLACK {
      self.num_wins += 1;
    } else if !black_wins && self.player == stone::WHITE {
      self.num_wins += 1;
    }
    return black_wins;
  }

  fn best_child(&mut self, num_sims: u32, uct_constant: f64) -> &mut Node {
    let mut best_value = -1f64;
    let mut best_child = 0;
    for i in 0 .. self.children.len() {
      let value = self.children[i].uct(num_sims, uct_constant);
      if value > best_value {
        best_value = value;
        best_child = i;
      }
    }
    &mut self.children[best_child]
  }

  pub fn uct(&self, num_sims: u32, uct_constant: f64) -> f64 {
    self.num_wins as f64 / self.num_plays as f64 +
        uct_constant * ((num_sims as f64).ln() / self.num_plays as f64).sqrt()
  }
}
