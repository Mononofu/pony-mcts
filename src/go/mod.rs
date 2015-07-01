extern crate rand;

use rand::Rng;
use std::fmt;
use std::collections;

use bench;

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

#[derive(Eq, Hash, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub struct Vertex {
  pub x: usize,
  pub y: usize,
}

impl fmt::Display for Vertex {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let column_labels = "ABCDEFGHIKLMNOPORSTU";
    try!(write!(f, "{}", column_labels.chars().nth(self.x).unwrap()));
    return write!(f, "{}", self.y + 1);
  }
}
impl fmt::Debug for Vertex {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let column_labels = "ABCDEFGHIKLMNOPORSTU";
    try!(write!(f, "{}", column_labels.chars().nth(self.x).unwrap()));
    return write!(f, "{}", self.y + 1);
  }
}


#[derive(Clone)]
struct String {
  color: Stone,
  stones: Vec<Vertex>,
  liberties: Vec<Vertex>,
}

#[derive(Clone)]
pub struct GoGame {
  size: usize,
  board: Vec<Vec<Option<Stone>>>,
  vertex_hashes: Vec<u64>,
  past_position_hashes: collections::HashSet<u64>,
  strings: collections::HashMap<u64, String>,
  string_index: Vec<Vec<u64>>,
  next_string_key: u64,
  last_single_capture: Option<Vertex>,
  timer: bench::Timer,
}

impl GoGame {
  pub fn new(size: usize) -> GoGame {
    let mut rng = rand::thread_rng();
    let mut vertex_hashes = vec![0; 3 * size * size];


    for col in 0 .. size {
      for row in 0 .. size {
        vertex_hashes[0 * size * size + col + row * size] = rng.gen(); // None
        vertex_hashes[1 * size * size + col + row * size] = rng.gen(); // Black
        vertex_hashes[2 * size * size + col + row * size] = rng.gen(); // White
      }
    }

    GoGame {
      size: size,
      board: vec![vec![None; size]; size],
      vertex_hashes: vertex_hashes,
      past_position_hashes: collections::HashSet::new(),
      strings: collections::HashMap::new(),
      string_index: vec![vec![0; size]; size],
      next_string_key: 1,
      last_single_capture: None,
      timer: bench::Timer::new(),
    }
  }

  pub fn vertex(&self, x: usize, y: usize) -> Vertex {
    Vertex {
      x: x,
      y: y,
    }
  }

  pub fn report(&self) {
    self.timer.report();
  }

  pub fn play(&mut self, stone: Stone, vertex: Vertex, force: bool) -> bool {
    self.timer.start("play");
    if !force && !self.can_play(stone, vertex) {
      return false;
    }

    self.last_single_capture = None;

    let mut liberties = Vec::new();
    for n in self.neighbours(vertex) {
      if self.stone_at(n) == None {
        liberties.push(n);
      }
    }
    liberties.sort();

    let string_key = self.next_string_key;
    self.next_string_key += 1;
    self.string_index[vertex.y][vertex.x] = string_key;
    self.strings.insert(string_key, String{
      color: stone,
      stones: vec![vertex],
      liberties: liberties,
    });

    self.board[vertex.y][vertex.x] = Some(stone);
    // Remove the vertex now occupied by this stone from the neighbor's liberties.
    for n in self.neighbours(vertex) {
      self.stone_at(n).map(|_| {
        let liberties = &mut self.strings.entry(self.string_index[n.y][n.x]).or_insert_with(|| panic!()).liberties;
        match liberties.binary_search(&vertex) {
          Ok(i) => { liberties.remove(i); () },
          Err(_) => (),
        };
      });
    }

    let mut single_captures = vec![];
    for n in self.neighbours(vertex) {
      self.stone_at(n).map(|s| {
        if s == stone.opponent() && self.dead(n) {
          if self.string(n).stones.len() == 1 {
            single_captures.push(n);
          }
          self.remove_group(n);
        }
      });
    }
    if single_captures.len() == 1 {
      self.last_single_capture = Some(single_captures[0]);
    }

    for n in self.neighbours(vertex) {
      self.stone_at(n).map(|s| {
         if s == stone {
          if self.string(n).stones.len() > self.string(vertex).stones.len() {
            self.join_groups(vertex, n);
          } else {
            self.join_groups(n, vertex);
          }
        }
      });
    }

    let hash = self.position_hash();
    if !force && self.past_position_hashes.contains(&hash) {
      println!("missed ko!");
    }
    self.past_position_hashes.insert(hash);
    self.timer.end();
    return true;
  }

  fn string(&self, vertex: Vertex) -> &String {
    return &self.strings[&self.string_index[vertex.y][vertex.x]];
  }

  fn join_groups(&mut self, smaller: Vertex, larger: Vertex) {
    self.timer.start("join_groups");
    let string_index = self.string_index[larger.y][larger.x];
    let smaller_string_index = self.string_index[smaller.y][smaller.x];

    if string_index == smaller_string_index {
      return;
    }

    for l in self.string(smaller).liberties.clone() {
      match self.strings[&string_index].liberties.binary_search(&l) {
        Ok(_) => (),
        Err(i) => self.strings.entry(string_index).or_insert_with(|| panic!()).liberties.insert(i, l),
      };
    }

    for v in self.group(smaller) {
      self.string_index[v.y][v.x] = string_index;
      self.strings.entry(string_index).or_insert_with(|| panic!()).stones.push(v);
    }

    self.strings.remove(&smaller_string_index);
    self.timer.end();
  }

  fn liberties(&self, vertex: Vertex) -> &Vec<Vertex> {
    return &self.string(vertex).liberties;
  }

  fn dead(&self, vertex: Vertex) -> bool {
    return self.string_index[vertex.y][vertex.x] == 0 ||
        self.string(vertex).liberties.len() == 0;
  }

  fn remove_group(&mut self, vertex: Vertex) {
    self.timer.start("remove_group");
    let string_index = self.string_index[vertex.y][vertex.x];
    for v in self.group(vertex) {
      self.board[v.y][v.x] = None;
      self.string_index[v.y][v.x] = 0;

      for n in self.neighbours(v) {
        self.stone_at(n).map(|_| {
          let neighbour_string_i = self.string_index[n.y][n.x];
          if (neighbour_string_i != string_index) {
            let liberties = &mut self.strings.entry(neighbour_string_i).or_insert_with(|| panic!("remove_group")).liberties;
            match liberties.binary_search(&v) {
              Ok(_) => (),
              Err(i) => liberties.insert(i, v),
            };
          }
        });
      }
    }
    self.strings.remove(&string_index);
    self.timer.end();
  }

  fn group(&self, vertex: Vertex) -> Vec<Vertex> {
    return self.string(vertex).stones.clone();
  }

  pub fn stone_at(&self, vertex: Vertex) -> Option<Stone> {
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

  pub fn can_play(&mut self, stone: Stone, vertex: Vertex) -> bool {
    self.timer.start("can_play");
    // Can't play if the vertex is not empty.
    if !self.board[vertex.y][vertex.x].is_none() {
      self.timer.end();
      return false;
    }

    // Can definitely play if the placed stone will have at least one direct
    // freedom (can't be ko).
    for n in self.neighbours(vertex) {
      if self.stone_at(n).is_none() {
        self.timer.end();
        return true;
      }
    }

    // For all checks below, the newly placed stone is completely surrounded by
    // enemy and friendly stones.

    // Don't allow to destroy real eyes.
    let ns = self.neighbours(vertex);
    if self.stone_at(ns[0]) == Some(stone) {
      let mut real_eye = true;
      let string_index = self.string_index[ns[0].y][ns[0].x];
      for n in ns {
        if string_index != self.string_index[n.y][n.x] {
          real_eye = false;
        }
      }
      if real_eye {
        self.timer.end();
        return false;
      }
    }

    // Allow to play if the placed stones connects to a group that still has at
    // least one other liberty after connecting.
    for n in self.neighbours(vertex) {
      if self.stone_at(n) == Some(stone) && self.string(n).liberties.len() > 1 {
        self.timer.end();
        return true;
      }
    }

    // Detect ko.
    if let Some(v) = self.last_single_capture {
      if v == vertex {
        return false;
      }
    }

    // Allow to play if the placed stone will kill at least one group.
    for n in self.neighbours(vertex) {
      if self.stone_at(n) == Some(stone.opponent()) && self.string(n).liberties.len() == 1 &&
          self.string(n).liberties.first() == Some(&vertex) {
        self.timer.end();
        return true;
      }
    }

    self.timer.end();

    // Don't allow to play if the stone would be dead or kill its own group.
    return false;
  }

  pub fn empty_vertices(&self) -> Vec<Vertex>  {
    let mut moves = Vec::new();
    for row in 0 .. self.size {
      for col in 0 .. self.size {
        let v = self.vertex(row, col);
        if self.board[v.y][v.x].is_none() {
          moves.push(v);
        }
      }
    }
    return moves;
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

  pub fn position_hash(&mut self) -> u64 {
    self.timer.start("position_hash");
    let mut hash = 0;
    for row in 0 .. self.size {
      for col in 0 .. self.size {
        let offset = match self.stone_at(self.vertex(col, row)) {
          None => 0,
          Some(Stone::Black) => 1,
          Some(Stone::White) => 2,
        };
        hash = hash ^ self.vertex_hashes[offset * self.size * self.size + col + row * self.size];
      }
    }
    self.timer.end();
    return hash;
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

#[cfg(test)]
mod test;
