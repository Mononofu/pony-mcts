extern crate log;
extern crate rand;

use rand::Rng;
use std::fmt;
use std::collections;

pub mod stone;
pub use self::stone::Stone;

pub mod vertex;
pub use self::vertex::Vertex;
pub use self::vertex::PASS;

pub mod constants;
pub use self::constants::NEIGHBOURS;
pub use self::constants::DIAG_NEIGHBOURS;

pub mod string;
pub use self::string::String;

// Maximum supported board size (width/height).
const MAX_SIZE: u8 = 19;
// Size of the virtual board necessary to support a board of MAX_SIZE.
// This includes a one stone border on all sides of the board.
const VIRT_SIZE: u8 = MAX_SIZE + 2;
// Length of an array/vector necessary to store the virtual board.
pub const VIRT_LEN: usize = VIRT_SIZE as usize * VIRT_SIZE as usize;


#[derive(Clone)]
pub struct GoGame {
  size: usize,
  // Board of stones with a 1-stone border on all sides to remove the need for
  // bound checking. Laid out as 1D vector, see GoGame::vertex for index
  // calculation.
  board: Vec<Stone>,

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
  // WHITE stones can be deduced from this, board size and empty_vertices.
  num_black_stones: i16,

  // Vertex that can't be played on because it would be simple ko.
  ko_vertex: Vertex,

  pub to_play: Stone,
  pub history: Vec<Vertex>,
}

impl GoGame {
  pub fn new(size: usize) -> GoGame {
    if size as u8 > MAX_SIZE {
      panic!("{} is larger than maximum supported board size of {}",
        size, MAX_SIZE);
    }

    let mut game = GoGame {
      size: size,
      board: vec![stone::BORDER; VIRT_LEN],

      strings: vec![String::new(); VIRT_LEN],
      string_head: vec![PASS; VIRT_LEN],
      string_next_v: vec![PASS; VIRT_LEN],

      empty_vertices: Vec::with_capacity(size * size),
      empty_v_index: vec![0; VIRT_LEN],

      num_black_stones: 0,

      ko_vertex: PASS,

      to_play: stone::BLACK,
      history: Vec::with_capacity(600),
    };
    game.reset();
    game
  }

  // Resets the game and clears the board. Same result as creating a new
  // instance, but this doesn't need to allocate any memory.
  pub fn reset(&mut self) {
    self.empty_vertices.clear();
    self.num_black_stones = 0;
    self.ko_vertex = PASS;
    self.to_play = stone::BLACK;
    self.history.clear();

    for i in 0 .. (VIRT_LEN) as usize {
      self.strings[i].reset_border();
      self.string_head[i] = Vertex(i as i16);
      self.string_next_v[i] = PASS;
    }

    for col in 0 .. self.size {
      for row in 0 .. self.size {
        let v = GoGame::vertex(row as i16, col as i16);
        self.board[v.as_index()] = stone::EMPTY;
        self.strings[v.as_index()].reset();

        self.empty_v_index[v.as_index()] = self.empty_vertices.len();
        self.empty_vertices.push(v);
      }
    }

    for col in 0 .. self.size {
      for row in 0 .. self.size {
        let v = Vertex::new(row as i16, col as i16);
        for n in NEIGHBOURS[v.as_index()].iter() {
          if self.stone_at(*n) == stone::EMPTY {
            self.strings[v.as_index()].add_liberty(*n);
          }
        }
      }
    }
  }

  pub fn vertex(x: i16, y: i16) -> Vertex {
    Vertex::new(x, y)
  }

  fn set_stone(&mut self, stone: Stone, vertex: Vertex) {
    let old_stone = self.board[vertex.as_index()];
    // Place new stone..
    self.board[vertex.as_index()] = stone;

    // Update empty vertex list.
    if stone == stone::EMPTY {
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
    if old_stone == stone::BLACK {
      self.num_black_stones -= 1;
    } else if stone == stone::BLACK {
      self.num_black_stones += 1;
    }
  }

  pub fn play(&mut self, stone: Stone, vertex: Vertex) -> bool {
    if cfg!(debug) && !self.can_play(stone, vertex) {
      return false;
    }

    self.to_play = stone.opponent();
    self.history.push(vertex);

    if vertex == PASS {
      return true;
    }

    // Preparation for ko checking.
    let old_num_empty_vertices = self.empty_vertices.len();
    let mut played_in_enemy_eye = true;
    for n in NEIGHBOURS[vertex.as_index()].iter() {
      let s = self.stone_at(*n);
      if s == stone || s == stone::EMPTY {
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

    return true;
  }

  pub fn undo(&mut self, num_moves: usize) -> bool {
    if num_moves > self.history.len() {
      return false;
    }
    let history = self.history.clone();
    self.reset();
    for i in 0 .. num_moves {
      let to_play = self.to_play;
      self.play(to_play, history[i]);
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
      if self.stone_at(*n) == stone::EMPTY {
        self.strings[largest_group_head.as_index()].add_liberty(*n);
      }
    }
  }

  fn init_new_string(&mut self, vertex: Vertex) {
    self.strings[vertex.as_index()].reset();
    self.strings[vertex.as_index()].num_stones += 1;

    for n in NEIGHBOURS[vertex.as_index()].iter() {
      if self.stone_at(*n) == stone::EMPTY {
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
      self.set_stone(stone::EMPTY, cur);
      let next = self.string_next_v[cur.as_index()];
      self.init_new_string(cur);

      for n in NEIGHBOURS[cur.as_index()].iter() {
        let neighbour_string_head = self.string_head[n.as_index()];
        if neighbour_string_head != string_head || self.stone_at(*n) == stone::EMPTY {
          self.strings[neighbour_string_head.as_index()].add_liberty(cur);
        }
      }

      cur = next;
      if cur == vertex {
        break;
      }
    }
  }

  pub fn stone_at(&self, vertex: Vertex) -> Stone {
    return self.board[vertex.as_index()]
  }

  pub fn can_play(&self, stone: Stone, vertex: Vertex) -> bool {
    if vertex == PASS {
      return true;
    }

    // Can't play if the vertex is not empty or would be ko.
    if self.stone_at(vertex) != stone::EMPTY || vertex == self.ko_vertex {
      return false;
    }

    // Can definitely play if the placed stone will have at least one direct
    // freedom (can't be ko).
    if self.string(vertex).num_pseudo_liberties > 0 {
      return true;
    }

    // For all checks below, the newly placed stone is completely surrounded by
    // enemy and friendly stones.

    // Don't allow to destroy eye-like points.
    let mut surrounded_by_own = true;
    let opponent = stone.opponent();
    for n in NEIGHBOURS[vertex.as_index()].iter() {
      let s = self.stone_at(*n);
      if s == opponent || s == stone::EMPTY {
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
        } else if s == stone::BORDER {
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

  pub fn possible_moves(&self, stone: Stone) -> Vec<Vertex> {
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
        if s == stone::BLACK {
          num_black += 1;
        } else if s == stone::WHITE {
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
    let column_labels = "ABCDEFGHJKLMNOPORST";
    try!(write!(f, "\x1b[0;37m    "));
    for col in 0 .. self.size {
      try!(write!(f, " {}", column_labels.chars().nth(col).unwrap()));
    }
    try!(write!(f, "\n"));

    for row in 0 .. self.size {
      try!(write!(f, " {:2} \x1b[43m\x1b[1;37m ", row + 1));
      for col in 0 .. self.size {
        try!(match self.stone_at(GoGame::vertex(col as i16, row as i16)) {
          stone::BLACK => write!(f, "\x1b[30m\u{25CF}\x1b[37m "),
          stone::WHITE => write!(f, "\u{25CF} "),
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


impl fmt::Debug for GoGame {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    let column_labels = "ABCDEFGHJKLMNOPORST";
    try!(write!(f, "    "));
    for col in 0 .. self.size {
      try!(write!(f, "{}", column_labels.chars().nth(col).unwrap()));
    }
    try!(write!(f, "\n"));

    let mut row = self.size - 1;
    loop {
      try!(write!(f, " {:2} ", row + 1));
      for col in 0 .. self.size {
        try!(match self.stone_at(GoGame::vertex(col as i16, row as i16)) {
          stone::BLACK => write!(f, "#"),
          stone::WHITE => write!(f, "O"),
          _ => write!(f, " ")
        });
      }
      try!(write!(f, " {:2}\n", row + 1));
      if row == 0 {
        break;
      }
      row -= 1;
    }

    try!(write!(f, "    "));
    for col in 0 .. self.size {
      try!(write!(f, "{}", column_labels.chars().nth(col).unwrap()));
    }

    return write!(f, "");
  }
}

#[cfg(test)]
mod test;
