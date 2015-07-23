use super::Stone;
use super::stone;
use super::Vertex;

// A string is a number of directly connected stones of the same color
// (diagonal connections are not enough).
#[derive(Clone)]
pub struct String {
  color: Stone,
  pub num_stones: u16,

  pub num_pseudo_liberties: u8,
  liberty_vertex_sum: u16,
  liberty_vertex_sum_squared: u32,
}

impl String {
  pub fn new() -> String {
    String {
      color: stone::EMPTY,
      num_stones: 0,

      num_pseudo_liberties: 0,
      liberty_vertex_sum: 0,
      liberty_vertex_sum_squared: 0,
    }
  }

  pub fn reset(&mut self) {
    self.color = stone::EMPTY;
    self.num_stones = 0;
    self.num_pseudo_liberties = 0;
    self.liberty_vertex_sum = 0;
    self.liberty_vertex_sum_squared = 0;
  }

  // Special string value for the border of virtual stones surrounding the real
  // board that is available for playing.
  // This removes the need for bounds checking.
  pub fn reset_border(&mut self) {
    self.color = stone::EMPTY;
    self.num_stones = 0;
    // Need to have values big enough that they can never go below 0 even if
    // all liberties are removed.
    self.num_pseudo_liberties = 4;
    self.liberty_vertex_sum = 32768;
    self.liberty_vertex_sum_squared = 2147483648;
  }

  pub fn merge(&mut self, other: &String) {
    self.num_stones += other.num_stones;
    self.num_pseudo_liberties += other.num_pseudo_liberties;
    self.liberty_vertex_sum += other.liberty_vertex_sum;
    self.liberty_vertex_sum_squared += other.liberty_vertex_sum_squared;
  }

  pub fn in_atari(&self) -> bool {
    return self.num_pseudo_liberties as u32 * self.liberty_vertex_sum_squared  ==
      self.liberty_vertex_sum as u32 * self.liberty_vertex_sum as u32;
  }

  pub fn add_liberty(&mut self, vertex: Vertex) {
    self.num_pseudo_liberties += 1;
    self.liberty_vertex_sum += vertex.0 as u16;
    self.liberty_vertex_sum_squared += vertex.0 as u32 * vertex.0 as u32;
  }

  pub fn remove_liberty(&mut self, vertex: Vertex) {
    self.num_pseudo_liberties -= 1;
    self.liberty_vertex_sum -= vertex.0 as u16;
    self.liberty_vertex_sum_squared -= vertex.0 as u32 * vertex.0 as u32;
  }
}
