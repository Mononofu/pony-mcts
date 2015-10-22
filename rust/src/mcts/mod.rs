extern crate log;
extern crate rand;

use go::Vertex;
use go::PASS;
use go::GoGame;
use go::Stone;
use go::stone;
use go::VIRT_LEN;
use rand::Rng;

const EXPANSION_THRESHOLD: u32 = 8;
const NODE_PRIOR: u32 = 10;
const UCT_C: f64 = 1.4;
const RAVE_C: f64 = 0.0;
const RAVE_EQUIV: f64 = 3500.0;

pub struct Node {
  player: Stone,
  pub vertex: Vertex,
  pub children: Vec<Node>,

  num_plays: u32,
  num_wins: u32,
  num_rave_plays: u32,
  num_rave_wins: u32,
}

pub struct Controller {
  pub root: Node,
}

fn black_wins(game: &mut GoGame, last_move: Stone, rng: &mut rand::StdRng,
      amaf_color_map: &mut Vec<Stone>) -> bool {
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
      if amaf_color_map[v.as_index()] == stone::EMPTY {
        amaf_color_map[v.as_index()] = color_to_play;
      }
      game.play(color_to_play, v);
      num_consecutive_passes = 0;
    }
    if num_moves > 700 {
      warn!("too many moves!");
      return false;
    }
  }
  return game.chinese_score() * 2 > double_komi;
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
      // Map to store who played at which vertex first to update node values by AMAF.
      let mut amaf_color_map = vec![stone::EMPTY; VIRT_LEN];
      self.root.run_rollout(i, &mut rollout_game, rng, &mut amaf_color_map);
    }
    if self.root.children.is_empty() {
      return PASS;
    }
    self.root.best_move().vertex
  }
}

impl Node {
  fn new(player: Stone, vertex: Vertex) -> Node {
    Node {
      player: player,
      vertex: vertex,
      children: vec![],

      num_plays: NODE_PRIOR,
      num_wins: NODE_PRIOR / 2,
      num_rave_plays: 0,
      num_rave_wins: 0,
    }
  }

  // Returns whether black wins to update the win rate in parent nodes.
  fn run_rollout(&mut self, num_sims: u32, game: &mut GoGame,
      rng: &mut rand::StdRng, amaf_color_map: &mut Vec<Stone>) -> bool {
    game.play(self.player, self.vertex);
    self.num_plays += 1;
    if self.vertex != PASS && amaf_color_map[self.vertex.as_index()] == stone::EMPTY {
      amaf_color_map[self.vertex.as_index()] = self.player;
    }

    let black_wins = if self.children.is_empty() {
      if self.num_plays > EXPANSION_THRESHOLD {
        let opponent = self.player.opponent();
        for v in game.possible_moves(opponent) {
          self.children.push(Node::new(opponent, v));
        }
        rng.shuffle(&mut self.children);
      }

      black_wins(game, self.player, rng, amaf_color_map)
    } else {
      self.best_child(num_sims).run_rollout(num_sims, game, rng, amaf_color_map)
    };


    let wins = if black_wins && self.player == stone::BLACK ||
      !black_wins && self.player == stone::WHITE {
      1
    } else {
      0
    };
    self.num_wins += wins;

    for c in self.children.iter_mut() {
      if amaf_color_map[c.vertex.as_index()] == c.player {
        c.num_rave_plays += 1;
        c.num_rave_wins += 1 - wins; // Children are from the other perspective.
      }
    }
    return black_wins;
  }

  fn best_move(&self) -> &Node {
    let mut max_visits = 0;
    let mut best_child = 0;
    for i in 0 .. self.children.len() {
      if self.children[i].num_plays > max_visits {
        best_child = i;
        max_visits = self.children[i].num_plays;
      }
    }
    &self.children[best_child]
  }

  fn best_child(&mut self, num_sims: u32) -> &mut Node {
    let mut best_value = -1f64;
    let mut best_child = 0;
    for i in 0 .. self.children.len() {
      let value = self.children[i].rave_urgency();
      if value > best_value {
        best_value = value;
        best_child = i;
      }
    }
    &mut self.children[best_child]
  }

  pub fn uct(&self, num_sims: u32) -> f64 {
    self.num_wins as f64 / self.num_plays as f64 +
        UCT_C * ((num_sims as f64).ln() / self.num_plays as f64).sqrt() +
        RAVE_C * (self.num_rave_wins as f64 / self.num_rave_plays as f64)
  }

  fn rave_urgency(&self) -> f64 {
    let value = self.num_wins as f64 / self.num_plays as f64;
    if self.num_rave_plays == 0 {
      return value;
    }

    let rave_value = self.num_rave_wins as f64 / self.num_rave_plays as f64;
    let beta = self.num_rave_plays as f64 / (
      self.num_rave_plays as f64 + self.num_plays as f64 +
      (self.num_rave_plays + self.num_plays) as f64 / RAVE_EQUIV);
    return beta * rave_value + (1.0 - beta) * value

  }
}
