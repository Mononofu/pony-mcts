use std::fmt;
use std::collections;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Stone {
  Black,
  White,
}

impl Stone {
  fn opponent(self) -> Stone {
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

pub struct GoGame {
  size: usize,
  board: Vec<Vec<Option<Stone>>>,
}

impl GoGame {
  pub fn new(size: usize) -> GoGame {
    GoGame {
      size: size,
      board: vec![vec![None; size]; size],
    }
  }

  pub fn vertex(&self, x: usize, y: usize) -> Vertex {
    Vertex {
      x: x,
      y: y,
    }
  }

  pub fn play(&mut self, stone: Stone, vertex: Vertex) -> bool {
    self.board[vertex.y][vertex.x] = Some(stone);
    for n in self.neighbours(vertex) {
      self.stone_at(n).map(|s| {
        if s == stone.opponent() && self.dead(n) {
          self.remove_group(n);
        }
      });
    }
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
