extern crate log;
extern crate rand;
extern crate vec_map;

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


#[derive(Copy, Clone)]
pub struct PosHash(usize);

impl PosHash {
  pub fn as_hash(self) -> usize {
    return self.0 as usize;
  }
}

pub struct BoardHasher {
  // Zobrist hashing for tracking super-ko and debugging normal ko checking.
  vertex_hashes: Vec<usize>,
  size: usize,
}

impl BoardHasher {
  pub fn new(size: usize) -> BoardHasher {
    let mut rng = rand::thread_rng();
    let mut vertex_hashes =  vec![0; 3 * VIRT_LEN];
    for col in 0 .. size {
      for row in 0 .. size {
        vertex_hashes[0 * size * size + col + row * size] = rng.gen(); // EMPTY
        vertex_hashes[1 * size * size + col + row * size] = rng.gen(); // BLACK
        vertex_hashes[2 * size * size + col + row * size] = rng.gen(); // WHITE
      }
    }

    return BoardHasher{
      vertex_hashes: vertex_hashes,
      size: size,
    };
  }

  pub fn hash(&self, game: &GoGame) -> PosHash {
    let mut hash: usize = 0;
    for col in 0 .. self.size {
      for row in 0 .. self.size {
        let v = Vertex::new(row as i16, col as i16);
        hash = hash ^ self.hash_for(v, game.stone_at(v));
      }
    }
    return PosHash(hash);
  }

  // Calculates zobrist hash for a vertex. Used for super-ko detection.
  fn hash_for(&self, vertex: Vertex, stone: Stone) -> usize {
    let offset = match stone {
      stone::EMPTY => 0,
      stone::BLACK => 1,
      stone::WHITE => 2,
      stone::BORDER => 3,
      _ => panic!("unknown stone"),
    };
    return self.vertex_hashes[offset * self.size * self.size + vertex.as_index()];
  }
}

pub struct Node {
  player: Stone,
  pub vertex: Vertex,
  pub children: Vec<PosHash>,

  num_plays: u32,
  num_wins: u32,
  num_rave_plays: u32,
  num_rave_wins: u32,
}

pub struct Controller {
  pub root: Node,
  nodes: vec_map::VecMap<Node>,
  hasher: BoardHasher,
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
  pub fn new(size: usize) -> Controller {
    Controller {
      root: Node::new(stone::WHITE, PASS),
      nodes: vec_map::VecMap::with_capacity(10000),
      hasher: BoardHasher::new(size),
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
      self.root.run_rollout(i, &mut rollout_game, rng, &mut amaf_color_map,
          &self.hasher, &mut self.nodes);
    }
    if self.root.children.is_empty() {
      return PASS;
    }
    self.root.best_move(&mut self.nodes).vertex
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
      rng: &mut rand::StdRng, amaf_color_map: &mut Vec<Stone>,
      hasher: &BoardHasher, nodes: &mut vec_map::VecMap<Node>) -> bool {
    game.play(self.player, self.vertex);
    self.num_plays += 1;
    if self.vertex != PASS && amaf_color_map[self.vertex.as_index()] == stone::EMPTY {
      amaf_color_map[self.vertex.as_index()] = self.player;
    }

    let black_wins = if self.children.is_empty() {
      if self.num_plays > EXPANSION_THRESHOLD {
        let opponent = self.player.opponent();
        for v in game.possible_moves(opponent) {
          game.play(opponent, v);
          let hash = hasher.hash(game);
          if !nodes.contains_key(&hash.as_hash()) {
            nodes.insert(hash.as_hash(), Node::new(opponent, v));
          }
          self.children.push(hash);
        }
        rng.shuffle(&mut self.children);
      }

      black_wins(game, self.player, rng, amaf_color_map)
    } else {
      self.best_child(num_sims, nodes).run_rollout(num_sims, game, rng,
          amaf_color_map, hasher, nodes)
    };


    let wins = if black_wins && self.player == stone::BLACK ||
      !black_wins && self.player == stone::WHITE {
      1
    } else {
      0
    };
    self.num_wins += wins;

    for hash in self.children.iter() {
      let ref mut child = nodes.get_mut(&hash.as_hash()).unwrap();
      if amaf_color_map[child.vertex.as_index()] == child.player {
        child.num_rave_plays += 1;
        child.num_rave_wins += 1 - wins; // Children are from the other perspective.
      }
    }
    return black_wins;
  }

  fn best_move<'a>(&'a self, nodes: &'a vec_map::VecMap<Node>) -> &Node {
    let mut max_visits = 0;
    let mut best_child = 0;
    for i in 0 .. self.children.len() {
      let child = nodes.get(&self.children[i].as_hash()).unwrap();
      if child.num_plays > max_visits {
        best_child = i;
        max_visits = child.num_plays;
      }
    }
    nodes.get(&self.children[best_child].as_hash()).unwrap()
  }

  fn best_child<'a>(&'a mut self, num_sims: u32, nodes: &'a mut vec_map::VecMap<Node>) -> &mut Node {
    let mut best_value = -1f64;
    let mut best_child = 0;
    for i in 0 .. self.children.len() {
      let child = nodes.get(&self.children[i].as_hash()).unwrap();
      let value = child.rave_urgency();
      if value > best_value {
        best_value = value;
        best_child = i;
      }
    }
    nodes.get_mut(&self.children[best_child].as_hash()).unwrap()
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
