use std::fmt;
use std::str;
use std::string;

// Enum struct with static lookup table for opponent is faster than a simple
// with a match x {} based function.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Stone(u8);

pub const EMPTY: Stone = Stone(0);
pub const BLACK: Stone = Stone(1);
pub const WHITE: Stone = Stone(2);
pub const BORDER: Stone = Stone(3);

pub static OPPONENT: [Stone; 4] = [EMPTY, WHITE, BLACK, BORDER];

impl Stone {
  pub fn opponent(self) -> Stone {
    return OPPONENT[self.0 as usize];
  }
}

impl fmt::Display for Stone {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    return write!(f, "{}", match self.0 {
      0 => "empty",
      1 => "black",
      2 => "white",
      3 => "border",
      _ => panic!(),
    });
  }
}

impl str::FromStr for Stone {
  type Err = string::String;

  fn from_str(s: &str) -> Result<Stone, string::String> {
    match s {
      "w" | "white" | "W" | "WHITE" => Ok(WHITE),
      "b" | "black" | "B" | "BLACK" => Ok(BLACK),
      _ => Err(format!("unknown color '{}'", s)),
    }
  }
}
