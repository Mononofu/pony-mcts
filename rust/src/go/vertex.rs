use std::fmt;
use std::str;
use std::string;

use super::VIRT_SIZE;

#[derive(Eq, Hash, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub struct Vertex(pub i16);

pub const PASS: Vertex = Vertex(-1);

impl Vertex {
  pub fn new(x: i16, y: i16) -> Vertex {
    Vertex(x + 1 + (y + 1) * VIRT_SIZE as i16)
  }

  pub fn to_coords(self) -> (i16, i16) {
    return ((self.0 % VIRT_SIZE as i16) - 1, self.0 / VIRT_SIZE as i16 - 1);
  }

  pub fn as_index(self) -> usize {
    return self.0 as usize;
  }
}

impl fmt::Display for Vertex {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    if *self == PASS {
      return write!(f, "PASS");
    }
    let (x, y) = self.to_coords();
    let column_labels = "aABCDEFGHJKLMNOPQRSTu";
    try!(write!(f, "{}", column_labels.chars().nth((x + 1) as usize).unwrap()));
    return write!(f, "{}", y + 1);
  }
}
impl fmt::Debug for Vertex {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    return write!(f, "{}", self);
  }
}

impl str::FromStr for Vertex {
  type Err = string::String;

  fn from_str(s: &str) -> Result<Vertex, string::String> {
    if s == "PASS" || s == "pass" {
      return Ok(PASS);
    }

    if s.len() < 2 || s.len() > 3 {
      return Err("expected Vertex of format A1".to_string());
    }
    let column_labels = "ABCDEFGHJKLMNOPQRST";
    let col_char = s.chars().next().unwrap();
    let col = match column_labels.find(|c| c == col_char) {
      Some(i) => i,
      None => return Err("column must be A - T".to_string()),
    };
    let row = (s[1..]).parse::<i16>();
    if row.is_err() {
      return Err("row must be integer".to_string());
    }

    Ok(Vertex::new(col as i16, row.unwrap() - 1))
  }
}
