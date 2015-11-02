extern crate log;
extern crate rand;

use go::Vertex;
use go::PASS;
use go::GoGame;
use go::Stone;
use go::stone;
use go::VIRT_LEN;
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::cmp;
use std::collections;
use std::cell;
use std::ops::Index;

mod zobrist;
use self::zobrist::BoardHasher;
use self::zobrist::PosHash;

#[cfg(test)]
mod test;

const NODE_PRIOR: u32 = 10;
const EXPANSION_THRESHOLD: u32 = 8 + NODE_PRIOR;
const UCT_C: f64 = 1.4;
const RAVE_C: f64 = 0.0;
const RAVE_EQUIV: f64 = 3500.0;

#[derive(Clone)]
pub struct Node {
  player: Stone,
  pub children: Vec<(Vertex, PosHash)>,
  parents: Vec<PosHash>,

  num_plays: u32,
  num_wins: u32,
  num_rave_plays: u32,
  num_rave_wins: u32,
}

struct NodeTable {
  nodes: Vec<(cell::Cell<PosHash>, cell::UnsafeCell<Node>)>,
  size: AtomicUsize,
}

impl NodeTable {
  fn with_capacity(c: usize) -> NodeTable {
    let mut table = NodeTable {
      nodes: vec![],
      size: AtomicUsize::new(0),
    };

    for _ in 0 .. c {
      table.nodes.push((cell::Cell::new(PosHash::None),
                        cell::UnsafeCell::new(Node::new(stone::EMPTY))));
    }

    return table;
  }

  fn get_mut(&self, hash: &PosHash) -> &mut Node {
    match self.find(hash) {
      Ok(i) => unsafe {
        let p_mut: *mut Node = self.nodes[i].1.get();
        &mut *p_mut
      },
      Err(_) => panic!("no entry for {:?}", hash),
    }
  }

  fn contains_key(&self, hash: &PosHash) -> bool {
    return self.find(hash).is_ok();
  }

  fn insert(&self, hash: PosHash, node: Node) {
    if self.size.load(Ordering::SeqCst) + 1 == self.nodes.len() {
      // Always leave at least one empty guard value.
      panic!("NodeTable is already full!");
    }

    match self.find(&hash) {
      Ok(_) => panic!("{:?} is already in the table", hash),
      Err(i) => unsafe {
        let p_mut:*mut Node = self.nodes[i].1.get();
        *p_mut = node;
        self.nodes[i].0.set(hash);
      },
    }

    self.size.fetch_add(1, Ordering::SeqCst);
  }

  // Returns either the position of the value for the hash, or the position
  // where it should be inserted.
  fn find(&self, hash: &PosHash) -> Result<usize, usize> {
    // This hash table uses linear probing.
    let start = hash.as_index() % self.nodes.len();
    match self.nodes[start].0.get() {
      PosHash::None => return Err(start),
      h if h == *hash => return Ok(start),
      _ => {},
    }

    let mut i = (start + 1) % self.nodes.len();
    while i != start {
      match self.nodes[i].0.get() {
        PosHash::None => return Err(i),
        h if h == *hash => return Ok(i),
        _ => i = (i + 1) % self.nodes.len(),
      }
    }

    panic!("table is completely full");
  }
}

impl Index<PosHash> for NodeTable {
  type Output = Node;

  fn index<'a>(&'a self, _index: PosHash) -> &'a Node {
    return self.get_mut(&_index);
  }
}

pub struct Controller {
  pub root: Node,
  nodes: NodeTable,
  hasher: BoardHasher,
}

fn black_wins(game: &mut GoGame, last_move: Stone, rng: &mut rand::StdRng,
      amaf_color_map: &mut Vec<Stone>) -> bool {
  let double_komi = 13;
  let mut color_to_play = last_move;
  let mut num_consecutive_passes = 0;
  let mut num_moves = 0;

  while num_consecutive_passes < 2 {
    // println!("{:?}", game);
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
      root: Node::new(stone::WHITE),
      nodes: NodeTable::with_capacity(100000),
      hasher: BoardHasher::new(),
    }
  }

  pub fn gen_move(&mut self, game: &GoGame, num_rollouts: u32, rng: &mut rand::StdRng) -> Vertex {
    let mut rollout_game = game.clone();
    if rollout_game.possible_moves(game.to_play).is_empty() {
      return PASS;
    }

    let root_hash = self.hasher.hash(game);

    if self.nodes.contains_key(&root_hash) {
      info!("reusing root with {:?} visits", self.nodes[root_hash].num_plays)
    } else {
      info!("creating a new root");
      self.nodes.insert(root_hash, Node::new(game.to_play));
    }
    {
      let mut root = self.nodes.get_mut(&root_hash);
      if root.children.is_empty() {
        self.expand_node(root_hash, &mut root, &mut rollout_game);
      }
    }

    for i in 1 .. num_rollouts + 1 {
      rollout_game.reset();
      for v in game.history.iter() {
        rollout_game.play(v.0, v.1);
      }
      self.run_rollout(i, root_hash, &mut rollout_game, rng);
    }

    self.print_statistics(root_hash);
    let (best_v, best_h) = self.nodes[root_hash].best_move(&self.nodes);
    info!("selected move {:}", best_v);
    self.print_statistics(best_h);

    return best_v;
  }

  fn run_rollout(&mut self, num_sims: u32, root_hash: PosHash, game: &mut GoGame,
      rng: &mut rand::StdRng) {
    // Map to store who played at which vertex first to update node values by AMAF.
    let mut amaf_color_map = vec![stone::EMPTY; VIRT_LEN];
    let mut hash = root_hash;
    let mut node = self.nodes.get_mut(&hash);

    // Run the simulation down the tree until we reach a leaf node.
    while !node.children.is_empty() {
      // Shuffle to break ties, todo(swj): find a faster way to break ties.
      rng.shuffle(&mut node.children);
      let (vertex, best_hash) = node.best_child(num_sims, &self.nodes);
      game.play(node.player, vertex);

      if vertex != PASS && amaf_color_map[vertex.as_index()] == stone::EMPTY {
        amaf_color_map[vertex.as_index()] = node.player;
      }

      hash = best_hash;
      node = self.nodes.get_mut(&hash);

      // Expand nodes with no children that are above the threshold.
      if node.children.is_empty() && node.num_plays > EXPANSION_THRESHOLD {
        self.expand_node(hash, node, game);
      }
    }

    // Run a random rollout till the end of the game.
    let black_wins = black_wins(game, node.player, rng, &mut amaf_color_map);

    // Propagate the new value up the tree, following all possible parent paths.
    let mut update_nodes = vec![hash];
    while !update_nodes.is_empty() {
      node = self.nodes.get_mut(&update_nodes.pop().unwrap());
      update_nodes.extend(node.parents.clone());

      let wins = if black_wins && node.player == stone::BLACK ||
          !black_wins && node.player == stone::WHITE {
        1
      } else {
        0
      };
      node.num_plays += 1;
      node.num_wins += wins;

      // Update the rave visits of all child nodes.
      for &(vertex, hash) in node.children.iter() {
        let ref mut child = self.nodes.get_mut(&hash);
        if amaf_color_map[vertex.as_index()] == child.player {
          child.num_rave_plays += 1;
          child.num_rave_wins += 1 - wins; // Children are from the other perspective.
        }
      }
    }
  }

  fn expand_node(&self, hash: PosHash, node: &mut Node, game: &mut GoGame) {
    let opponent = node.player.opponent();
    for v in game.possible_moves(opponent) {
      game.play(opponent, v);
      let child_hash = self.hasher.hash(game);
      game.undo(1);
      if !self.nodes.contains_key(&child_hash) {
        self.nodes.insert(child_hash, Node::new(opponent));
      }
      // Add this node as parent to its new children.
      self.nodes.get_mut(&child_hash).parents.push(hash);
      node.children.push((v, child_hash));
    }
  }

  fn print_statistics(&self, root_hash: PosHash) {
    let ref root = self.nodes[root_hash];
    info!("node hash: {:?}", root_hash);

    let mut children = root.children.clone();
    children.sort_by(|a, b| self.nodes[b.1].num_plays.cmp(
        &self.nodes[a.1].num_plays));
    for i in 0 .. cmp::min(10, children.len()) {
      let (vertex, hash) = children[i];
      let ref child = self.nodes[hash];
      info!("{:?}: {:} visits {:?}", vertex, child.num_plays, hash);
    }

    self.print_pv(root_hash);
  }

  fn print_pv(&self, root_hash: PosHash) {
    let mut hash = root_hash;
    let mut node = self.nodes.get_mut(&hash);
    let mut pv = vec![];

    while !node.children.is_empty() {
      let (vertex, hash) = node.best_move(&self.nodes);
      node = self.nodes.get_mut(&hash);
      pv.push((vertex, node.num_plays));
    }

    info!("PV: {:?}", pv);
  }
}

impl Node {
  fn new(player: Stone) -> Node {
    Node {
      player: player,
      children: vec![],
      parents: vec![],

      num_plays: NODE_PRIOR,
      num_wins: NODE_PRIOR / 2,
      num_rave_plays: 0,
      num_rave_wins: 0,
    }
  }

  fn best_move(&self, nodes: &NodeTable) -> (Vertex, PosHash) {
    let mut max_visits = 0;
    let mut best_child = 0;
    for i in 0 .. self.children.len() {
      let num_plays = nodes[self.children[i].1].num_plays;
      if num_plays > max_visits {
        best_child = i;
        max_visits = num_plays;
      }
    }
    return self.children[best_child];
  }

  fn best_child(&self, num_sims: u32, nodes: &NodeTable) -> (Vertex, PosHash) {
    let mut best_value = -1f64;
    let mut best_child = 0;
    for i in 0 .. self.children.len() {
      let value = nodes[self.children[i].1].rave_urgency();
      if value > best_value {
        best_value = value;
        best_child = i;
      }
    }
    return self.children[best_child];
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
