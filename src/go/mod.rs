extern crate rand;

use rand::Rng;
use std::fmt;
use std::collections;

use bench;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Stone {
  Empty,
  Black,
  White,
  Border,
}

impl Stone {
  pub fn opponent(self) -> Stone {
    match self {
      Stone::Empty => Stone::Empty,
      Stone::Black => Stone::White,
      Stone::White => Stone::Black,
      Stone::Border => Stone::Border,
    }
  }
}

#[derive(Eq, Hash, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub struct Vertex(pub u16);

impl Vertex {
  fn to_coords(self) -> (u16, u16) {
    let Vertex(v) = self;
    return (v % 21 - 1, v / 21 - 1);
  }
}

impl fmt::Display for Vertex {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let (x, y) = self.to_coords();
    let column_labels = "ABCDEFGHIKLMNOPORSTU";
    try!(write!(f, "{}", column_labels.chars().nth(x as usize).unwrap()));
    return write!(f, "{}", y + 1);
  }
}
impl fmt::Debug for Vertex {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let (x, y) = self.to_coords();
    let column_labels = "ABCDEFGHIKLMNOPORSTU";
    try!(write!(f, "{}", column_labels.chars().nth(x as usize).unwrap()));
    return write!(f, "{}", y + 1);
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
  board: Vec<Stone>,
  vertex_hashes: Vec<u64>,
  past_position_hashes: collections::HashSet<u64>,
  position_hash: u64,
  strings: collections::HashMap<u64, String>,
  string_index: Vec<u64>,
  next_string_key: u64,
  last_single_capture: Option<Vertex>,
  pub timer: bench::Timer,
}

impl GoGame {
  pub fn new(size: usize) -> GoGame {
    let mut rng = rand::thread_rng();


    let mut board = vec![Stone::Border; 21 * 21];
    let mut hash = 0;
    let mut vertex_hashes = vec![0; 3 * board.len()];
    for col in 0 .. size {
      for row in 0 .. size {
        vertex_hashes[0 * size * size + col + row * size] = rng.gen(); // Empty
        vertex_hashes[1 * size * size + col + row * size] = rng.gen(); // Black
        vertex_hashes[2 * size * size + col + row * size] = rng.gen(); // White
        // Create initial board hash.
        hash = hash ^ vertex_hashes[0 * size * size + col + row * size];
        let Vertex(v) = GoGame::vertex(row as u16, col as u16);
        board[v as usize] = Stone::Empty;
      }
    }

    GoGame {
      size: size,
      board: board,
      vertex_hashes: vertex_hashes,
      past_position_hashes: collections::HashSet::with_capacity(500),
      position_hash: hash,
      strings: collections::HashMap::with_capacity(100),
      string_index: vec![0; 21 * 21],
      next_string_key: 1,
      last_single_capture: None,
      timer: bench::Timer::new(),
    }
  }

  pub fn vertex(x: u16, y: u16) -> Vertex {
    Vertex(x + 1 + (y + 1) * 21)
  }

  fn hash_for(&self, vertex: Vertex) -> u64 {
    let offset = match self.stone_at(vertex) {
      Stone::Empty => 0,
      Stone::Black => 1,
      Stone::White => 2,
      Stone::Border => 3,
    };
    let Vertex(v) = vertex;
    return self.vertex_hashes[offset * self.size * self.size + v as usize];
  }

  fn set_stone(&mut self, stone: Stone, vertex: Vertex) {
    // Remove hash for old stone.
    self.position_hash = self.position_hash ^ self.hash_for(vertex);
    // Place new stone and apply hash for it.
    let Vertex(v) = vertex;
    self.board[v as usize] = stone;
    self.position_hash = self.position_hash ^ self.hash_for(vertex);
  }

  pub fn play(&mut self, stone: Stone, vertex: Vertex, force: bool) -> bool {
    self.timer.start("play");
    if !force && !self.can_play(stone, vertex) {
      return false;
    }

    self.timer.section("insert string");

    self.last_single_capture = None;

    let mut liberties = Vec::new();
    for n in self.neighbours(vertex) {
      if self.stone_at(n) == Stone::Empty {
        liberties.push(n);
      }
    }
    liberties.sort();

    let string_key = self.next_string_key;
    self.next_string_key += 1;
    let Vertex(v) = vertex;
    self.string_index[v as usize] = string_key;
    self.strings.insert(string_key, String{
      color: stone,
      stones: vec![vertex],
      liberties: liberties,
    });

    self.timer.section("join groups");

    self.check_neighbours_for_joining(vertex, stone);

    self.timer.section("remove liberties");

    self.set_stone(stone, vertex);
    // Remove the vertex now occupied by this stone from the neighbor's liberties.
    for Vertex(n) in self.neighbours(vertex) {
      match self.stone_at(Vertex(n)) {
        Stone::Black | Stone::White => {
          let liberties = &mut self.strings.entry(self.string_index[n as usize]).or_insert_with(|| panic!()).liberties;
          match liberties.binary_search(&vertex) {
            Ok(i) => { liberties.remove(i); () },
            Err(_) => (),
          };
        },
        _ => (),
      }
    }

    self.timer.section("capture groups");

    let mut single_capture = None;
    let mut num_captures = 0;
    for n in self.neighbours(vertex) {
      if self.stone_at(n) == stone.opponent() && self.dead(n) {
        if self.string(n).stones.len() == 1 {
          single_capture = Some(n);
        }
        num_captures += 1;
        self.remove_group(n);
      }
    }
    if num_captures == 1 {
      self.last_single_capture = single_capture;
    }

    self.timer.section("check ko");

    if !force && self.past_position_hashes.contains(&self.position_hash) {
      println!("missed ko!");
    }
    self.past_position_hashes.insert(self.position_hash);
    self.timer.end();
    return true;
  }

  fn check_neighbours_for_joining(&mut self, vertex: Vertex, stone: Stone) {
    self.timer.start("check_neighbours_for_joining");
    for n in self.neighbours(vertex) {
      if self.stone_at(n) == stone {
        if self.string(n).stones.len() > self.string(vertex).stones.len() {
          self.join_groups(vertex, n);
        } else {
          self.join_groups(n, vertex);
        }
      }
    }
    self.timer.end();
  }

  fn string(&self, vertex: Vertex) -> &String {
    let Vertex(v) = vertex;
    return &self.strings[&self.string_index[v as usize]];
  }

  fn join_groups(&mut self, smaller: Vertex, larger: Vertex) {
    self.timer.start("join_groups");
    let Vertex(l) = larger;
    let string_index = self.string_index[l as usize];
    let Vertex(s) = smaller;
    let smaller_string_index = self.string_index[s as usize];

    if string_index == smaller_string_index {
      self.timer.end();
      return;
    }

    for l in self.string(smaller).liberties.clone() {
      match self.strings[&string_index].liberties.binary_search(&l) {
        Ok(_) => (),
        Err(i) => self.strings.entry(string_index).or_insert_with(|| panic!()).liberties.insert(i, l),
      };
    }

    for Vertex(v) in self.group(smaller) {
      self.string_index[v as usize] = string_index;
      self.strings.entry(string_index).or_insert_with(|| panic!()).stones.push(Vertex(v));
    }

    self.strings.remove(&smaller_string_index);
    self.timer.end();
  }

  fn liberties(&self, vertex: Vertex) -> &Vec<Vertex> {
    return &self.string(vertex).liberties;
  }

  fn dead(&self, vertex: Vertex) -> bool {
    let Vertex(v) = vertex;
    return self.string_index[v as usize] == 0 ||
        self.string(vertex).liberties.len() == 0;
  }

  fn remove_group(&mut self, vertex: Vertex) {
    self.timer.start("remove_group");
    let Vertex(v) = vertex;
    let string_index = self.string_index[v as usize];
    for Vertex(v) in self.group(vertex) {
      self.set_stone(Stone::Empty, Vertex(v));
      self.string_index[v as usize] = 0;

      for Vertex(n) in self.neighbours(Vertex(v)) {
        let stone = self.stone_at(Vertex(n));
        if stone == Stone::White || stone == Stone::Black {
          let neighbour_string_i = self.string_index[n as usize];
          if neighbour_string_i != string_index {
            let liberties = &mut self.strings.entry(neighbour_string_i).or_insert_with(|| panic!("remove_group")).liberties;
            match liberties.binary_search(&Vertex(v)) {
              Ok(_) => (),
              Err(i) => liberties.insert(i, Vertex(v)),
            };
          }
        }
      }
    }
    self.strings.remove(&string_index);
    self.timer.end();
  }

  fn group(&self, vertex: Vertex) -> Vec<Vertex> {
    return self.string(vertex).stones.clone();
  }

  pub fn stone_at(&self, vertex: Vertex) -> Stone {
    let Vertex(v) = vertex;
    return self.board[v as usize]
  }

  fn neighbours(&self, vertex: Vertex) -> Vec<Vertex> {
    let Vertex(v) = vertex;
    return vec![Vertex(v - 1), Vertex(v + 1), Vertex(v - 21), Vertex(v + 21)];
  }

  pub fn can_play(&mut self, stone: Stone, vertex: Vertex) -> bool {
    self.timer.start("can_play");
    // Can't play if the vertex is not empty.
    if self.stone_at(vertex) != Stone::Empty {
      self.timer.end();
      return false;
    }

    // Can definitely play if the placed stone will have at least one direct
    // freedom (can't be ko).
    for n in self.neighbours(vertex) {
      if self.stone_at(n) == Stone::Empty {
        self.timer.end();
        return true;
      }
    }

    // For all checks below, the newly placed stone is completely surrounded by
    // enemy and friendly stones.

    // Don't allow to destroy real eyes.
    let ns = self.neighbours(vertex);
    if self.stone_at(ns[0]) == stone {
      let mut real_eye = true;
      let Vertex(ns0) = ns[0];
      let string_index = self.string_index[ns0 as usize];
      for Vertex(n) in ns {
        if string_index != self.string_index[n as usize] {
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
      if self.stone_at(n) == stone && self.string(n).liberties.len() > 1 {
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
      if self.stone_at(n) == stone.opponent() && self.string(n).liberties.len() == 1 &&
          self.string(n).liberties.first() == Some(&vertex) {
        self.timer.end();
        return true;
      }
    }

    self.timer.end();

    // Don't allow to play if the stone would be dead or kill its own group.
    return false;
  }

  pub fn empty_vertices(&mut self) -> Vec<Vertex>  {
    self.timer.start("empty_vertices");
    let mut moves = Vec::new();
    for row in 0 .. self.size {
      for col in 0 .. self.size {
        let vertex = GoGame::vertex(row as u16, col as u16);
        let Vertex(v) = vertex;
        if self.board[v as usize] == Stone::Empty {
          moves.push(vertex);
        }
      }
    }
    self.timer.end();
    return moves;
  }

  pub fn possible_moves(&mut self, stone: Stone) -> Vec<Vertex> {
    let mut moves = Vec::new();
    for row in 0 .. self.size {
      for col in 0 .. self.size {
        let v = GoGame::vertex(row as u16, col as u16);
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
        try!(match self.stone_at(GoGame::vertex(row as u16, col as u16)) {
          Stone::Black => write!(f, "\x1b[30m\u{25CF}\x1b[37m "),
          Stone::White => write!(f, "\u{25CF} "),
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
