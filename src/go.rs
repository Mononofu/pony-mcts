extern crate rand;

use rand::Rng;
use std::fmt;
use std::collections;
use std::hash;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Stone {
  Black,
  White,
}

impl Stone {
  pub fn opponent(self) -> Stone {
    match self {
      Stone::Black => Stone::White,
      Stone::White => Stone::Black,
    }
  }
}

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub struct Vertex {
  pub x: usize,
  pub y: usize,
}

#[derive(Clone)]
pub struct GoGame {
  size: usize,
  board: Vec<Vec<Option<Stone>>>,
  vertex_hashes: collections::HashMap<(Option<Stone>, Vertex), u64>,
  past_position_hashes: collections::HashSet<u64>,
}

impl GoGame {
  pub fn new(size: usize) -> GoGame {
    let mut rng = rand::thread_rng();
    let mut vertex_hashes = collections::HashMap::new();

    for col in 0 .. size {
      for row in 0 .. size {
        vertex_hashes.insert((None, Vertex{x: col, y: row}), rng.gen());
        vertex_hashes.insert((Some(Stone::Black), Vertex{x: col, y: row}), rng.gen());
        vertex_hashes.insert((Some(Stone::White), Vertex{x: col, y: row}), rng.gen());
      }
    }

    GoGame {
      size: size,
      board: vec![vec![None; size]; size],
      vertex_hashes: vertex_hashes,
      past_position_hashes: collections::HashSet::new(),
    }
  }

  pub fn vertex(&self, x: usize, y: usize) -> Vertex {
    Vertex {
      x: x,
      y: y,
    }
  }

  pub fn play(&mut self, stone: Stone, vertex: Vertex, force: bool) -> bool {
    if !force && !self.can_play(stone, vertex) {
      return false;
    }

    self.board[vertex.y][vertex.x] = Some(stone);
    for n in self.neighbours(vertex) {
      self.stone_at(n).map(|s| {
        if s == stone.opponent() && self.dead(n) {
          self.remove_group(n);
        }
      });
    }

    let mut hasher = hash::SipHasher::new();
    self.hash(&mut hasher);
    self.past_position_hashes.insert(hasher.finish());
    return true;
  }

  fn dead(&self, vertex: Vertex) -> bool {
    for v in self.group(vertex) {
      for n in self.neighbours(v) {
        if self.stone_at(n).is_none() {
          return false;
        }
      }
    }
    return true;
  }

  fn remove_group(&mut self, vertex: Vertex) {
    for v in self.group(vertex) {
      self.board[v.y][v.x] = None
    }
  }

  fn group(&self, vertex: Vertex) -> collections::HashSet<Vertex> {
    let mut g = collections::HashSet::new();
    let mut candidates = vec![vertex];
    while !candidates.is_empty() {
      let v = candidates.pop().unwrap();
      g.insert(v);
      for n in self.neighbours(v) {
        if self.stone_at(v) == self.stone_at(n) {
          if !g.contains(&n) {
            g.insert(n);
            candidates.push(n);
          }
        }
      }
    }
    return g;
  }

  fn stone_at(&self, vertex: Vertex) -> Option<Stone> {
    return self.board[vertex.y][vertex.x]
  }

  fn neighbours(&self, v: Vertex) -> Vec<Vertex> {
    let mut ns = Vec::new();
    if v.x > 0 {
      ns.push(self.vertex(v.x - 1, v.y));
    }
    if v.y > 0 {
      ns.push(self.vertex(v.x, v.y - 1));
    }
    if v.x + 1 < self.size {
      ns.push(self.vertex(v.x  + 1, v.y));
    }
    if v.y + 1 < self.size {
      ns.push(self.vertex(v.x, v.y + 1));
    }
    return ns;
  }

  fn can_play(&mut self, stone: Stone, vertex: Vertex) -> bool {
    // Can't play if the vertex is not empty.
    if !self.board[vertex.y][vertex.x].is_none() {
      return false;
    }

    // Detect ko.
    let mut playout = self.clone();
    playout.play(stone, vertex, true);

    let mut hasher = hash::SipHasher::new();
    playout.hash(&mut hasher);
    if self.past_position_hashes.contains(&hasher.finish()) {
      // This board position already happened previously - ko!
      return false
    }

    // Can definitely play if the placed stone will have at least one direct
    // freedom,
    for n in self.neighbours(vertex) {
      if self.stone_at(n).is_none() {
        return true;
      }
    }

    // Don't allow to destroy real eyes.
    let ns = self.neighbours(vertex);
    if self.stone_at(ns[0]) == Some(stone) {
      let mut real_eye = true;
      let g = self.group(ns[0]);
      for n in ns {
        if !g.contains(&n) {
          real_eye = false;
        }
      }
      if real_eye {
        return false;
      }
    }

    // Allow to play if the placed stone will kill at least one group.
    self.board[vertex.y][vertex.x] = Some(stone);
    for n in self.neighbours(vertex) {
      if self.stone_at(n) == Some(stone.opponent()) && self.dead(n) {
        self.board[vertex.y][vertex.x] = None;
        return true;
      }
    }

    // Don't allow to play if the stone would be dead or kill its own group.
    let can_play = !self.dead(vertex);
    self.board[vertex.y][vertex.x] = None;
    return can_play;
  }

  pub fn possible_moves(&mut self, stone: Stone) -> Vec<Vertex> {
    let mut moves = Vec::new();
    for row in 0 .. self.size {
      for col in 0 .. self.size {
        let v = self.vertex(row, col);
        if self.can_play(stone, v) {
          moves.push(v);
        }
      }
    }
    return moves;
  }
}

impl fmt::Display for GoGame {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let column_labels = "ABCDEFGHIKLMNOPORSTU";
    try!(write!(f, "\x1b[0;37m    "));
    for col in 0 .. self.size {
      try!(write!(f, " {}", column_labels.chars().nth(col).unwrap()));
    }
    try!(write!(f, "\n"));

    for row in 0 .. self.size {
      try!(write!(f, " {:2} \x1b[43m\x1b[1;37m ", row + 1));
      for col in 0 .. self.size {
        try!(match self.board[row][col] {
          Some(Stone::Black) => write!(f, "\x1b[30m\u{25CF}\x1b[37m "),
          Some(Stone::White) => write!(f, "\u{25CF} "),
          _ => write!(f, "\u{00b7} ")
        });
      }
      try!(write!(f, "\x1b[0;37m {:2}\n", row + 1));
    }

    try!(write!(f, "    "));
    for col in 0 .. self.size {
      try!(write!(f, " {}", column_labels.chars().nth(col).unwrap()));
    }

    return write!(f, "");
  }
}

impl hash::Hash for GoGame {
  fn hash<H: hash::Hasher>(&self, state: &mut H) {
    for row in 0 .. self.size {
      for col in 0 .. self.size {
        let v = self.vertex(col, row);
        self.vertex_hashes.get(&(self.stone_at(v), v)).unwrap().hash(state);
      }
    }
  }
}
