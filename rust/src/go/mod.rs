extern crate rand;

use rand::Rng;
use std::fmt;
use std::collections;

// Enum struct with static lookup table for opponent is faster than a simple
// with a match x {} based function.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Stone(u8);

pub mod Stone {
  use super::Stone;

  pub const Empty: Stone = Stone(0);
  pub const Black: Stone = Stone(1);
  pub const White: Stone = Stone(2);
  pub const Border: Stone = Stone(3);

  pub static OPPONENT: [Stone; 4] = [Empty, White, Black, Border];

  impl Stone {
    pub fn opponent(self) -> Stone {
      return OPPONENT[self.0 as usize];
    }
  }
}

mod constants;
pub use self::constants::NEIGHBOURS;
pub use self::constants::DIAG_NEIGHBOURS;
pub use self::constants::Vertex;
pub use self::constants::PASS;

// Maximum supported board size (width/height).
const MAX_SIZE: u8 = 19;
// Size of the virtual board necessary to support a board of MAX_SIZE.
// This includes a one stone border on all sides of the board.
const VIRT_SIZE: u8 = MAX_SIZE + 2;
// Length of an array/vector necessary to store the virtual board.
const VIRT_LEN: usize = VIRT_SIZE as usize * VIRT_SIZE as usize;

impl Vertex {
  fn to_coords(self) -> (i16, i16) {
    return ((self.0 % VIRT_SIZE as i16) - 1, self.0 / VIRT_SIZE as i16 - 1);
  }

  fn as_index(self) -> usize {
    return self.0 as usize;
  }
}

impl fmt::Display for Vertex {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let (x, y) = self.to_coords();
    let column_labels = "aABCDEFGHIKLMNOPORSTUu";
    try!(write!(f, "{}", column_labels.chars().nth((x + 1) as usize).unwrap()));
    return write!(f, "{}", y + 1);
  }
}
impl fmt::Debug for Vertex {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    return write!(f, "{}", self);
  }
}

// A string is a number of directly connected stones of the same color
// (diagonal connections are not enough).
#[derive(Clone)]
struct String {
  color: Stone,
  num_stones: u16,

  num_pseudo_liberties: u8,
  liberty_vertex_sum: u16,
  liberty_vertex_sum_squared: u32,
}

impl String {
  fn reset(&mut self) {
    self.color = Stone::Empty;
    self.num_stones = 0;
    self.num_pseudo_liberties = 0;
    self.liberty_vertex_sum = 0;
    self.liberty_vertex_sum_squared = 0;
  }

  // Special string value for the border of virtual stones surrounding the real
  // board that is available for playing.
  // This removes the need for bounds checking.
  fn reset_border(&mut self) {
    self.color = Stone::Empty;
    self.num_stones = 0;
    // Need to have values big enough that they can never go below 0 even if
    // all liberties are removed.
    self.num_pseudo_liberties = 4;
    self.liberty_vertex_sum = 32768;
    self.liberty_vertex_sum_squared = 2147483648;
  }

  fn merge(&mut self, other: &String) {
    self.num_stones += other.num_stones;
    self.num_pseudo_liberties += other.num_pseudo_liberties;
    self.liberty_vertex_sum += other.liberty_vertex_sum;
    self.liberty_vertex_sum_squared += other.liberty_vertex_sum_squared;
  }

  fn in_atari(&self) -> bool {
    return self.num_pseudo_liberties as u32 * self.liberty_vertex_sum_squared  ==
      self.liberty_vertex_sum as u32 * self.liberty_vertex_sum as u32;
  }

  fn add_liberty(&mut self, vertex: Vertex) {
    self.num_pseudo_liberties += 1;
    self.liberty_vertex_sum += vertex.0 as u16;
    self.liberty_vertex_sum_squared += vertex.0 as u32 * vertex.0 as u32;
  }

  fn remove_liberty(&mut self, vertex: Vertex) {
    self.num_pseudo_liberties -= 1;
    self.liberty_vertex_sum -= vertex.0 as u16;
    self.liberty_vertex_sum_squared -= vertex.0 as u32 * vertex.0 as u32;
  }
}

pub struct GoGame {
  size: usize,
  // Board of stones with a 1-stone border on all sides to remove the need for
  // bound checking. Laid out as 1D vector, see GoGame::vertex for index
  // calculation.
  board: Vec<Stone>,

  // Zobrist hashing for tracking super-ko and debugging normal ko checking.
  // Disabled in release mode for better performance.
  vertex_hashes: Vec<u64>,
  past_position_hashes: collections::HashSet<u64>,
  position_hash: u64,

  strings: Vec<String>,
  // Head of the string for every vertex of the board.
  string_head: Vec<Vertex>,
  // Implicit representation of the linked list of all stones belonging to the
  // same String. Cyclic, indexed by Vertex.
  string_next_v: Vec<Vertex>,

  // Vector of all empty vertices.
  empty_vertices: Vec<Vertex>,
  // Position of every vertex in the vector above, to allow constant time
  // removal and addition.
  empty_v_index: Vec<usize>,

  // Number of black stones on the board, for scoring at the end of the game.
  // White stones can be deduced from this, board size and empty_vertices.
  num_black_stones: i16,

  // Vertex that can't be played on because it would be simple ko.
  ko_vertex: Vertex,
}

impl GoGame {
  pub fn new(size: usize) -> GoGame {
    if size as u8 > MAX_SIZE {
      panic!("{} is larger than maximum supported board size of {}",
        size, MAX_SIZE);
    }

    let mut rng = rand::thread_rng();

    let mut board = vec![Stone::Border; VIRT_LEN];
    let mut hash = 0;
    let mut vertex_hashes =  if cfg!(debug) { vec![0; 3 * board.len()] } else { vec![] };
    let mut empty_vertices = Vec::with_capacity(size * size);
    let mut empty_v_index = vec![0; VIRT_LEN];
    for col in 0 .. size {
      for row in 0 .. size {
        if cfg!(debug) {
          vertex_hashes[0 * size * size + col + row * size] = rng.gen(); // Empty
          vertex_hashes[1 * size * size + col + row * size] = rng.gen(); // Black
          vertex_hashes[2 * size * size + col + row * size] = rng.gen(); // White
          // Create initial board hash.
          hash = hash ^ vertex_hashes[0 * size * size + col + row * size];
        }

        let v = GoGame::vertex(row as i16, col as i16);
        board[v.as_index()] = Stone::Empty;

        empty_v_index[v.as_index()] = empty_vertices.len();
        empty_vertices.push(v);
      }
    }


    let mut string_head = vec![PASS; VIRT_LEN];
    for i in 0 .. VIRT_LEN {
      string_head[i] = Vertex(i as i16);
    }

    let strings = vec![String{
      color: Stone::Empty,
      num_stones: 0,

      num_pseudo_liberties: 4,
      liberty_vertex_sum: 32768, // 2 ^ 15
      liberty_vertex_sum_squared: 2147483648, // 2 ^ 31
    }; VIRT_LEN];

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
      string_head: string_head,
      string_next_v: vec![PASS; VIRT_LEN],

      empty_vertices: empty_vertices,
      empty_v_index: empty_v_index,

      num_black_stones: 0,

      ko_vertex: PASS,
    }
  }

  // Resets the game and clears the board. Same result as creating a new
  // instance, but this doesn't need to allocate any memory.
  pub fn reset(&mut self) {
    self.empty_vertices.clear();
    self.past_position_hashes.clear();
    self.num_black_stones = 0;
    self.ko_vertex = PASS;

    for i in 0 .. (VIRT_LEN) as usize {
      self.strings[i].reset_border();
      self.string_head[i] = Vertex(i as i16);
      self.string_next_v[i] = PASS;
    }

    let mut hash = 0;

    for col in 0 .. self.size {
      for row in 0 .. self.size {
        if cfg!(debug) {
          hash = hash ^ self.vertex_hashes[0 * self.size * self.size + col + row * self.size];
        }

        let v = GoGame::vertex(row as i16, col as i16);
        self.board[v.as_index()] = Stone::Empty;

        self.empty_v_index[v.as_index()] = self.empty_vertices.len();
        self.empty_vertices.push(v);
      }
    }

    self.position_hash = hash;
  }

  pub fn vertex(x: i16, y: i16) -> Vertex {
    Vertex(x + 1 + (y + 1) * VIRT_SIZE as i16)
  }

  // Calculates zobrist hash for a vertex. Used for super-ko detection.
  fn hash_for(&self, vertex: Vertex) -> u64 {
    let offset = match self.stone_at(vertex) {
      Stone::Empty => 0,
      Stone::Black => 1,
      Stone::White => 2,
      Stone::Border => 3,
      _ => panic!("unknown stone"),
    };
    return self.vertex_hashes[offset * self.size * self.size + vertex.as_index()];
  }

  fn set_stone(&mut self, stone: Stone, vertex: Vertex) {
    let old_stone = self.board[vertex.as_index()];
    // Remove hash for old stone.
    if cfg!(debug) {
      self.position_hash = self.position_hash ^ self.hash_for(vertex);
    }
    // Place new stone and apply hash for it.
    self.board[vertex.as_index()] = stone;
    if cfg!(debug) {
      self.position_hash = self.position_hash ^ self.hash_for(vertex);
    }

    // Update empty vertex list.
    if stone == Stone::Empty {
      self.empty_v_index[vertex.as_index()] = self.empty_vertices.len();
      self.empty_vertices.push(vertex);
    } else {
      let i = self.empty_v_index[vertex.as_index()];
      {
        let last = self.empty_vertices.last().unwrap();
        self.empty_v_index[last.as_index()] = i;
      }
      self.empty_vertices.swap_remove(i);
    }

    // Update stone count for scoring.
    if old_stone == Stone::Black {
      self.num_black_stones -= 1;
    } else if stone == Stone::Black {
      self.num_black_stones += 1;
    }
  }

  pub fn play(&mut self, stone: Stone, vertex: Vertex) -> bool {
    if cfg!(debug) && !self.can_play(stone, vertex) {
      return false;
    }

    // Preparation for ko checking.
    let old_num_empty_vertices = self.empty_vertices.len();
    let mut played_in_enemy_eye = true;
    for n in NEIGHBOURS[vertex.as_index()].iter() {
      let s = self.stone_at(*n);
      if s == stone || s == Stone::Empty {
        played_in_enemy_eye = false;
      }
    }
    self.ko_vertex = PASS;

    self.join_groups_around(vertex, stone);
    self.set_stone(stone, vertex);
    self.remove_liberty_from_neighbouring_groups(vertex);
    self.capture_dead_groups(vertex, stone);

    if played_in_enemy_eye && old_num_empty_vertices == self.empty_vertices.len() {
      self.ko_vertex = *self.empty_vertices.last().unwrap();
    }

    if cfg!(debug) {
      self.check_ko();
    }
    return true;
  }

  fn remove_liberty_from_neighbouring_groups(&mut self, vertex: Vertex) {
    for n in NEIGHBOURS[vertex.as_index()].iter() {
      self.strings[self.string_head[n.as_index()].as_index()].remove_liberty(vertex);
    }
  }

  fn capture_dead_groups(&mut self, vertex: Vertex, stone: Stone) {
    for n in NEIGHBOURS[vertex.as_index()].iter() {
      if self.stone_at(*n) == stone.opponent() && self.dead(*n) {
        self.remove_group(*n);
      }
    }
  }

  fn check_ko(&mut self) {
    if self.past_position_hashes.contains(&self.position_hash) {
      println!("missed ko!");
    }
    self.past_position_hashes.insert(self.position_hash);
  }

  fn string(&self, vertex: Vertex) -> &String {
    return &self.strings[self.string_head[vertex.as_index()].as_index()];
  }

  fn num_pseudo_liberties(&self, vertex: Vertex) -> u8 {
    return self.string(vertex).num_pseudo_liberties;
  }

  // Combines the groups around the newly placed stone at vertex. If no groups
  // are available for joining, the new stone is placed as it's one new group.
  fn join_groups_around(&mut self, vertex: Vertex, stone: Stone) {
    let mut largest_group_head = PASS;
    let mut largest_group_size = 0;
    for n in NEIGHBOURS[vertex.as_index()].iter() {
      if self.stone_at(*n) == stone {
        let string = self.string(*n);
        if string.num_stones > largest_group_size {
          largest_group_size = string.num_stones;
          largest_group_head = self.string_head[n.as_index()];
        }
      }
    }

    if largest_group_size == 0 {
      self.init_new_string(vertex);
      return;
    }

    for n in NEIGHBOURS[vertex.as_index()].iter() {
      if self.stone_at(*n) == stone {
        let string_head = self.string_head[n.as_index()];
        if string_head != largest_group_head {
          // Set all the stones in the smaller string to be part of the larger
          // string.
          let mut cur = *n;
          loop {
            self.string_head[cur.as_index()] = largest_group_head;
            cur = self.string_next_v[cur.as_index()];
            if cur == *n {
              break;
            }
          }

          // Connect the two linked lists representing the stones in the two
          // strings.
          let tmp = self.string_next_v[largest_group_head.as_index()];
          self.string_next_v[largest_group_head.as_index()] = self.string_next_v[n.as_index()];
          self.string_next_v[n.as_index()] = tmp;

          let (small, large) = (string_head.as_index(), largest_group_head.as_index());
          if small < large {
            let (left, right) = self.strings.split_at_mut(large);
            right[0].merge(&left[small]);
          } else {
            let (left, right) = self.strings.split_at_mut(small);
            left[large].merge(&right[0]);
          }
        }
      }
    }

    self.string_next_v[vertex.as_index()] = self.string_next_v[largest_group_head.as_index()];
    self.string_next_v[largest_group_head.as_index()] = vertex;
    self.strings[largest_group_head.as_index()].num_stones += 1;
    self.string_head[vertex.as_index()] = largest_group_head;

    for n in NEIGHBOURS[vertex.as_index()].iter() {
      if self.stone_at(*n) == Stone::Empty {
        self.strings[largest_group_head.as_index()].add_liberty(*n);
      }
    }
  }

  fn init_new_string(&mut self, vertex: Vertex) {
    self.strings[vertex.as_index()].reset();
    self.strings[vertex.as_index()].num_stones += 1;

    for n in NEIGHBOURS[vertex.as_index()].iter() {
      if self.stone_at(*n) == Stone::Empty {
        self.strings[vertex.as_index()].add_liberty(*n);
      }
    }

    self.string_head[vertex.as_index()] = vertex;
    self.string_next_v[vertex.as_index()] = vertex;
  }

  fn dead(&self, vertex: Vertex) -> bool {
    return self.string(vertex).num_pseudo_liberties == 0;
  }

  fn remove_group(&mut self, vertex: Vertex) {
    let mut cur = vertex;
    let string_head = self.string_head[vertex.as_index()];

    loop {
      self.set_stone(Stone::Empty, cur);
      self.string_head[cur.as_index()] = cur;
      self.strings[cur.as_index()].reset();

      for n in NEIGHBOURS[cur.as_index()].iter() {
        let stone = self.board[n.as_index()];

        if stone == Stone::White || stone == Stone::Black {
          let neighbour_string_head = self.string_head[n.as_index()];
          if neighbour_string_head != string_head {
            self.strings[neighbour_string_head.as_index()].add_liberty(cur);
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

  pub fn can_play(&self, stone: Stone, vertex: Vertex) -> bool {
    // Can't play if the vertex is not empty or would be ko.
    if self.stone_at(vertex) != Stone::Empty || vertex == self.ko_vertex {
      return false;
    }

    // Can definitely play if the placed stone will have at least one direct
    // freedom (can't be ko).
    for n in NEIGHBOURS[vertex.as_index()].iter() {
      if self.stone_at(*n) == Stone::Empty {
        return true;
      }
    }

    // For all checks below, the newly placed stone is completely surrounded by
    // enemy and friendly stones.

    // Don't allow to destroy eye-like points.
    let mut surrounded_by_own = true;
    let opponent = stone.opponent();
    for n in NEIGHBOURS[vertex.as_index()].iter() {
      let s = self.stone_at(*n);
      if s == opponent || s == Stone::Empty {
        surrounded_by_own = false;
        break;
      }
    }
    if surrounded_by_own {
      let mut enemy_count = 0;
      let mut border = 0;
      for n in DIAG_NEIGHBOURS[vertex.as_index()].iter() {
        let s = self.stone_at(*n);
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
    for n in NEIGHBOURS[vertex.as_index()].iter() {
      if self.stone_at(*n) == stone && !self.string(*n).in_atari() {
        return true;
      }
    }

    // Allow to play if the placed stone will kill at least one group.
    for n in NEIGHBOURS[vertex.as_index()].iter() {
      if self.stone_at(*n) == stone.opponent() && self.string(*n).in_atari() {
        return true;
      }
    }

    // Don't allow to play if the stone would be dead or kill its own group.
    return false;
  }

  pub fn random_move(&self, stone: Stone, rng: &mut rand::StdRng) -> Vertex {
    let num_empty = self.empty_vertices.len();
    let start_vertex = rng.gen_range(0, num_empty);
    let mut i = start_vertex;

    loop {
      let v = self.empty_vertices[i];
      if self.can_play(stone, v) {
        return v;
      }
      i += 1;
      if i == num_empty {
        i = 0;
      }
      if i == start_vertex {
        return PASS;
      }
    }
  }

  pub fn possible_moves(&mut self, stone: Stone) -> Vec<Vertex> {
    return self.empty_vertices.iter().map(|v| v.clone())
      .filter(|v| self.can_play(stone, *v)).collect::<Vec<_>>();
  }

  pub fn chinese_score(&self) -> i16 {
    let num_white_stones = (self.size * self.size) as i16 - self.num_black_stones - self.empty_vertices.len() as i16;

    let mut eye_score = 0;
    for v in self.empty_vertices.iter() {
      let mut num_black = 0;
      let mut num_white = 0;

      for n in NEIGHBOURS[v.as_index()].iter() {
        let s = self.stone_at(*n);
        if s == Stone::Black {
          num_black += 1;
        } else if s == Stone::White {
          num_white += 1;
        } else {
          num_black += 1;
          num_white += 1;
        }
      }

      if num_black == 4 {
        eye_score += 1;
      } else if num_white == 4 {
        eye_score -= 1;
      }
    }

    return self.num_black_stones - num_white_stones + eye_score;
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
