extern crate rand;

use rand::Rng;
use std::fmt;
use std::collections;
use std::mem;

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
pub struct Vertex(pub i16);

pub const PASS: Vertex = Vertex(-1);

impl Vertex {
  fn to_coords(self) -> (i16, i16) {
    return ((self.0 % 21) - 1, self.0 / 21 - 1);
  }

  fn as_index(self) -> usize {
    return self.0 as usize;
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
    return write!(f, "{}", self);
  }
}


#[derive(Clone)]
struct String {
  color: Stone,
  num_stones: u16,
  liberties: Vec<Vertex>,
}

pub struct GoGame {
  size: usize,
  board: Vec<Stone>,

  vertex_hashes: Vec<u64>,
  past_position_hashes: collections::HashSet<u64>,
  position_hash: u64,

  strings: Vec<String>,
  // Index of the string every Vertex belongs to.
  string_index: Vec<usize>,
  // Implicit representation of the linked list of all stones belonging to the
  // same String. Cyclic, indexed by Vertex.
  string_next_v: Vec<Vertex>,

  last_single_capture: Option<Vertex>,
}

impl GoGame {
  pub fn new(size: usize) -> GoGame {
    let mut rng = rand::thread_rng();

    let mut board = vec![Stone::Border; 21 * 21];
    let mut hash = 0;
    let mut vertex_hashes =  if cfg!(debug) { vec![0; 3 * board.len()] } else { vec![] };
    for col in 0 .. size {
      for row in 0 .. size {
        if cfg!(debug) {
          vertex_hashes[0 * size * size + col + row * size] = rng.gen(); // Empty
          vertex_hashes[1 * size * size + col + row * size] = rng.gen(); // Black
          vertex_hashes[2 * size * size + col + row * size] = rng.gen(); // White
          // Create initial board hash.
          hash = hash ^ vertex_hashes[0 * size * size + col + row * size];
        }
        board[GoGame::vertex(row as i16, col as i16).as_index()] = Stone::Empty;
      }
    }

    let mut strings = Vec::with_capacity(500);
    // Add a null string.
    strings.push(String{
      color: Stone::Empty,
      num_stones: 0,
      liberties: vec![],
    });

    let past_position_hashes = if cfg!(debug) {
      collections::HashSet::with_capacity(500)
    } else {
      collections::HashSet::new()
    };
    GoGame {
      size: size,
      board: board,
      vertex_hashes: vertex_hashes,
      past_position_hashes: past_position_hashes,
      position_hash: hash,
      strings: strings,
      string_index: vec![0; 21 * 21],
      string_next_v: vec![Vertex(0); 21 * 21],
      last_single_capture: None,
    }
  }

  pub fn vertex(x: i16, y: i16) -> Vertex {
    Vertex(x + 1 + (y + 1) * 21)
  }

  fn hash_for(&self, vertex: Vertex) -> u64 {
    let offset = match self.stone_at(vertex) {
      Stone::Empty => 0,
      Stone::Black => 1,
      Stone::White => 2,
      Stone::Border => 3,
    };
    return self.vertex_hashes[offset * self.size * self.size + vertex.as_index()];
  }

  fn set_stone(&mut self, stone: Stone, vertex: Vertex) {
    // Remove hash for old stone.
    if cfg!(debug) {
      self.position_hash = self.position_hash ^ self.hash_for(vertex);
    }
    // Place new stone and apply hash for it.
    self.board[vertex.as_index()] = stone;
    if cfg!(debug) {
      self.position_hash = self.position_hash ^ self.hash_for(vertex);
    }
  }

  pub fn play(&mut self, stone: Stone, vertex: Vertex) -> bool {
    if cfg!(debug) && !self.can_play(stone, vertex) {
      return false;
    }
    self.last_single_capture = None;
    self.join_groups_around(vertex, stone);
    self.set_stone(stone, vertex);
    self.remove_liberty_from_neighbouring_groups(vertex);
    self.capture_dead_groups(vertex, stone);
    if cfg!(debug) {
      self.check_ko();
    }
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

    self.string_index[vertex.as_index()] = self.strings.len();
    self.string_next_v[vertex.as_index()] = vertex;
    self.strings.push(String{
      color: stone,
      num_stones: 1,
      liberties: liberties,
    });
  }

  fn remove_liberty_from_neighbouring_groups(&mut self, vertex: Vertex) {
    for n in GoGame::neighbours(vertex) {
      let string_index = self.string_index[n.as_index()];
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
        if self.string(n).num_stones == 1 {
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
    return &self.strings[self.string_index[vertex.as_index()]];
  }

  fn join_groups_around(&mut self, vertex: Vertex, stone: Stone) {
    let mut largest_group_v = PASS;
    let mut largest_group_size = 0;
    let mut largest_group_i = -1;
    for n in GoGame::neighbours(vertex) {
      if self.stone_at(n) == stone {
        let string = self.string(n);
        if string.num_stones > largest_group_size {
          largest_group_size = string.num_stones;
          largest_group_v = n;
          largest_group_i = self.string_index[n.as_index()];
        }
      }
    }

    if largest_group_size == 0 {
      self.place_new_stone_as_string(vertex, stone);
      return;
    }

    for n in GoGame::neighbours(vertex) {
      if self.stone_at(n) == stone {
        let string_i = self.string_index[n.as_index()];
        if string_i != largest_group_i {
          let mut cur = n;
          loop {
            self.string_index[cur.as_index()] = largest_group_i;
            cur = self.string_next_v[cur.as_index()];
            if cur == n {
              break;
            }
          }
          let tmp = self.string_next_v[largest_group_v.as_index()];
          self.string_next_v[largest_group_v.as_index()] = self.string_next_v[n.as_index()];
          self.string_next_v[n.as_index()] = tmp;
          self.strings[largest_group_i].num_stones += self.strings[string_i].num_stones;

          let mut liberties = vec![];
          {
            use std::mem::swap;
            mem::swap(&mut self.strings[string_i].liberties, &mut liberties);
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

    self.string_next_v[vertex.as_index()] = self.string_next_v[largest_group_v.as_index()];
    self.string_next_v[largest_group_v.as_index()] = vertex;
    self.strings[largest_group_i].num_stones += 1;
    self.string_index[vertex.as_index()] = largest_group_i;

    for n in GoGame::neighbours(vertex) {
      if self.stone_at(n) == Stone::Empty {
        let mut libs = &mut self.strings[largest_group_i].liberties;
        match libs.binary_search(&n) {
          Ok(_) => (),
          Err(i) => libs.insert(i, n.clone()),
        };
      }
    }
  }

  fn liberties(&self, vertex: Vertex) -> &Vec<Vertex> {
    return &self.string(vertex).liberties;
  }

  fn dead(&self, vertex: Vertex) -> bool {
    return self.string_index[vertex.as_index()] == 0 ||
        self.string(vertex).liberties.len() == 0;
  }

  fn remove_group(&mut self, vertex: Vertex) {
    let mut cur = vertex;
    let string_index = self.string_index[vertex.as_index()];

    loop {
      self.set_stone(Stone::Empty, cur);
      self.string_index[cur.as_index()] = 0;

      for &n in GoGame::neighbours(cur).iter() {
        let stone = self.board[n.as_index()];

        if stone == Stone::White || stone == Stone::Black {
          let neighbour_string_i = self.string_index[n.as_index()];
          if neighbour_string_i != string_index {
            let liberties = &mut self.strings[neighbour_string_i].liberties;
            match liberties.binary_search(&cur) {
              Ok(_) => (),
              Err(i) => liberties.insert(i, cur),
            };
          }
        }
      }

      cur = self.string_next_v[cur.as_index()];
      if cur == vertex {
        break;
      }
    }
  }

  pub fn stone_at(&self, vertex: Vertex) -> Stone {
    return self.board[vertex.as_index()]
  }

  fn neighbours(v: Vertex) -> Vec<Vertex> {
    return vec![Vertex(v.0 - 1), Vertex(v.0 + 1), Vertex(v.0 - 21), Vertex(v.0 + 21)];
  }

  fn diag_neighbours(v: Vertex) -> Vec<Vertex> {
    return vec![Vertex(v.0 - 22), Vertex(v.0 - 20), Vertex(v.0 + 20), Vertex(v.0 + 22)];
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

    // Don't allow to destroy eye-like points.
    let mut surrounded_by_own = true;
    let opponent = stone.opponent();
    for Vertex(n) in GoGame::neighbours(vertex) {
      let s = self.stone_at(Vertex(n));
      if s == opponent || s == Stone::Empty {
        surrounded_by_own = false;
        break;
      }
    }
    if surrounded_by_own {
      let mut enemy_count = 0;
      let mut border = 0;
      for Vertex(n) in GoGame::diag_neighbours(vertex) {
        let s = self.stone_at(Vertex(n));
        if s == opponent {
          enemy_count += 1;
        } else if s == Stone::Border {
          border = 1;
        }
      }

      if enemy_count + border < 2 {
        // eye-like point
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
        let vertex = GoGame::vertex(row as i16, col as i16);
        if self.board[vertex.as_index()] == Stone::Empty {
          moves.push(vertex);
        }
      }
    }
    return moves;
  }

  pub fn random_move(&self, stone: Stone, rng: &mut rand::StdRng) -> Vertex {
    let num_vertices = 21 * (self.size as i16 + 1);
    let start_vertex = rng.gen_range(22, num_vertices);
    let mut v = start_vertex;

    loop {
      if self.can_play(stone, Vertex(v)) {
        return Vertex(v);
      }
      v += 1;
      if v == num_vertices {
        v = 0;
      }
      if v == start_vertex {
        return PASS;
      }
    }
  }

  pub fn possible_moves(&mut self, stone: Stone) -> Vec<Vertex> {
    let mut moves = Vec::new();
    for row in 0 .. self.size {
      for col in 0 .. self.size {
        let v = GoGame::vertex(row as i16, col as i16);
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
        try!(match self.stone_at(GoGame::vertex(col as i16, row as i16)) {
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
