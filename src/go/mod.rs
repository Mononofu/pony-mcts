extern crate rand;

use rand::Rng;
use std::fmt;
use std::collections;

const MAX_LIBERTIES_PER_STRING: usize = 10;
const MIN_LIBERTIES_PER_STRING: usize = 5;

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
    return ((v % 21) - 1, v / 21 - 1);
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
  stones: collections::LinkedList<Vertex>,
  liberties: Vec<Vertex>,
}

pub struct GoGame {
  size: usize,
  board: Vec<Stone>,
  vertex_hashes: Vec<u64>,
  past_position_hashes: collections::HashSet<u64>,
  position_hash: u64,
  strings: Vec<String>,
  string_index: Vec<usize>,
  last_single_capture: Option<Vertex>,
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

    let mut strings = Vec::with_capacity(500);
    // Add a null string.
    strings.push(String{
      color: Stone::Empty,
      stones: collections::LinkedList::new(),
      liberties: vec![],
    });

    GoGame {
      size: size,
      board: board,
      vertex_hashes: vertex_hashes,
      past_position_hashes: collections::HashSet::with_capacity(500),
      position_hash: hash,
      strings: strings,
      string_index: vec![0; 21 * 21],
      last_single_capture: None,
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

  pub fn play(&mut self, stone: Stone, vertex: Vertex) -> bool {
    if !self.can_play(stone, vertex) {
      return false;
    }
    self.last_single_capture = None;
    self.join_groups_around(vertex, stone);
    self.set_stone(stone, vertex);
    self.remove_liberty_from_neighbouring_groups(vertex);
    self.capture_dead_groups(vertex, stone);
    self.check_ko();
    return true;
  }

  fn place_new_stone_as_string(&mut self, vertex: Vertex, stone: Stone) {
    let mut liberties = Vec::new();
    for n in GoGame::neighbours(vertex) {
      if self.stone_at(n) == Stone::Empty {
        liberties.push(n);
      }
    }
    liberties.sort();

    let Vertex(v) = vertex;
    self.string_index[v as usize] = self.strings.len();
    let mut stones = collections::LinkedList::new();
    stones.push_back(vertex);
    self.strings.push(String{
      color: stone,
      stones: stones,
      liberties: liberties,
    });
  }

  fn remove_liberty_from_neighbouring_groups(&mut self, vertex: Vertex) {
    for Vertex(n) in GoGame::neighbours(vertex) {
      let string_index = self.string_index[n as usize];
      if string_index != 0 {
        let liberties = &mut self.strings[string_index].liberties;
        match liberties.binary_search(&vertex) {
          Ok(i) => { liberties.remove(i); () },
          Err(_) => (),
        };
      }
    }
  }

  fn capture_dead_groups(&mut self, vertex: Vertex, stone: Stone) {
    let mut single_capture = None;
    let mut num_captures = 0;
    for n in GoGame::neighbours(vertex) {
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
  }

  fn check_ko(&mut self) {
    if self.past_position_hashes.contains(&self.position_hash) {
      println!("missed ko!");
    }
    self.past_position_hashes.insert(self.position_hash);
  }

  fn string(&self, vertex: Vertex) -> &String {
    let Vertex(v) = vertex;
    return &self.strings[self.string_index[v as usize]];
  }

  fn join_groups_around(&mut self, vertex: Vertex, stone: Stone) {
    let mut largest_group_size = 0;
    let mut largest_group_i = -1;
    for n in GoGame::neighbours(vertex) {
      if self.stone_at(n) == stone {
        let string = self.string(n);
        if string.stones.len() > largest_group_size {
          largest_group_size = string.stones.len();
          let Vertex(v) = n;
          largest_group_i = self.string_index[v as usize];
        }
      }
    }

    if largest_group_size == 0 {
      self.place_new_stone_as_string(vertex, stone);
      return;
    }

    for n in GoGame::neighbours(vertex) {
      if self.stone_at(n) == stone {
        let Vertex(v) = n;
        let string_i = self.string_index[v as usize];
        if string_i != largest_group_i {
          let mut stones = collections::LinkedList::new();
          {
            use std::mem::swap;
            swap(&mut self.strings[string_i].stones, &mut stones);
          }
          for &Vertex(v) in stones.iter() {
            self.string_index[v as usize] = largest_group_i;
          }
          self.strings[largest_group_i].stones.append(&mut stones);

          let mut liberties = vec![];
          {
            use std::mem::swap;
            swap(&mut self.strings[string_i].liberties, &mut liberties);
          }

          for l in liberties.iter() {
            let mut libs = &mut self.strings[largest_group_i].liberties;
            match libs.binary_search(&l) {
              Ok(_) => (),
              Err(i) => libs.insert(i, l.clone()),
            };
          }
        }
      }
    }

    self.strings[largest_group_i].stones.push_back(vertex);
    for n in GoGame::neighbours(vertex) {
      if self.stone_at(n) == Stone::Empty {
        let mut libs = &mut self.strings[largest_group_i].liberties;
        match libs.binary_search(&n) {
          Ok(_) => (),
          Err(i) => libs.insert(i, n.clone()),
        };
      }
    }
    let Vertex(v) = vertex;
    self.string_index[v as usize] = largest_group_i;
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
    let Vertex(v) = vertex;
    let string_index = self.string_index[v as usize];
    // let mut stones = collections::LinkedList::new();
    // {
    //   use std::mem::swap;
    //   swap(&mut self.strings[string_index].stones, &mut stones);
    // }
    self.strings.push(String{
      color: Stone::Empty,
      stones: collections::LinkedList::new(),
      liberties: vec![],
    });
    let string = self.strings.swap_remove(string_index);

    for &Vertex(v) in string.stones.iter() {
      self.set_stone(Stone::Empty, Vertex(v));
      self.string_index[v as usize] = 0;

      for &Vertex(n) in GoGame::neighbours(Vertex(v)).iter() {
        let stone = self.board[n as usize];

        if stone == Stone::White || stone == Stone::Black {
          let neighbour_string_i = self.string_index[n as usize];
          if neighbour_string_i != string_index {
            let liberties = &mut self.strings[neighbour_string_i].liberties;
            match liberties.binary_search(&Vertex(v)) {
              Ok(_) => (),
              Err(i) => liberties.insert(i, Vertex(v)),
            };
          }
        }
      }
    }
  }

  pub fn stone_at(&self, vertex: Vertex) -> Stone {
    let Vertex(v) = vertex;
    return self.board[v as usize]
  }

  fn neighbours(vertex: Vertex) -> Vec<Vertex> {
    let Vertex(v) = vertex;
    return vec![Vertex(v - 1), Vertex(v + 1), Vertex(v - 21), Vertex(v + 21)];
  }

  pub fn can_play(&self, stone: Stone, vertex: Vertex) -> bool {
    // Can't play if the vertex is not empty.
    if self.stone_at(vertex) != Stone::Empty {
      return false;
    }

    // Detect ko.
    if let Some(v) = self.last_single_capture {
      if v == vertex {
        return false;
      }
    }

    // Can definitely play if the placed stone will have at least one direct
    // freedom (can't be ko).
    for n in GoGame::neighbours(vertex) {
      if self.stone_at(n) == Stone::Empty {
        return true;
      }
    }

    // For all checks below, the newly placed stone is completely surrounded by
    // enemy and friendly stones.

    // Don't allow to destroy real eyes.
    let ns = GoGame::neighbours(vertex);
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
        return false;
      }
    }

    // Allow to play if the placed stones connects to a group that still has at
    // least one other liberty after connecting.
    for n in GoGame::neighbours(vertex) {
      if self.stone_at(n) == stone && self.string(n).liberties.len() > 1 {
        return true;
      }
    }

    // Allow to play if the placed stone will kill at least one group.
    for n in GoGame::neighbours(vertex) {
      if self.stone_at(n) == stone.opponent() && self.string(n).liberties.len() == 1 &&
          self.string(n).liberties.first() == Some(&vertex) {
        return true;
      }
    }

    // Don't allow to play if the stone would be dead or kill its own group.
    return false;
  }

  pub fn empty_vertices(&mut self) -> Vec<Vertex>  {
    let mut moves = Vec::with_capacity(self.size * self.size);
    for row in 0 .. self.size {
      for col in 0 .. self.size {
        let vertex = GoGame::vertex(row as u16, col as u16);
        let Vertex(v) = vertex;
        if self.board[v as usize] == Stone::Empty {
          moves.push(vertex);
        }
      }
    }
    return moves;
  }

  pub fn random_move(&self, stone: Stone, rng: &mut rand::StdRng) -> Option<Vertex> {
    let num_vertices = self.board.len() as u16;
    let start_vertex = rng.gen_range(0, num_vertices);
    let mut v = start_vertex;

    while true {
      if self.can_play(stone, Vertex(v)) {
        return Some(Vertex(v));
      }
      v += 1;
      if (v == num_vertices) {
        v = 0;
      }
      if (v == start_vertex) {
        return None;
      }
    }

    return None;
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
        try!(match self.stone_at(GoGame::vertex(col as u16, row as u16)) {
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
