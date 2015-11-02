extern crate rand;

use std::ops::BitXor;
use std::ops::Rem;
use rand::Rng;

use go::VIRT_LEN;
use go::VIRT_SIZE;
use go::GoGame;
use go::Vertex;
use go::Stone;
use go::stone;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PosHash(u64);

impl BitXor for PosHash {
	type Output = PosHash;

	fn bitxor(self, rhs: PosHash) -> PosHash {
		PosHash(self.0 ^ rhs.0)
	}
}

impl PosHash {
  pub const None: PosHash = PosHash(0);

  pub fn new(h: u64) -> PosHash {
    PosHash(h)
  }

  pub fn as_index(self) -> usize {
    self.0 as usize
  }
}

pub struct BoardHasher {
  // Zobrist hashing for tracking super-ko and debugging normal ko checking.
  vertex_hashes: Vec<PosHash>,
}

impl BoardHasher {
  pub fn new() -> BoardHasher {
    let mut rng = rand::thread_rng();
    let mut vertex_hashes =  vec![PosHash(0); 3 * VIRT_LEN];
    let size = VIRT_SIZE as usize;
    for col in 0 .. size {
      for row in 0 .. size {
        vertex_hashes[0 * VIRT_LEN + col + row * size] = PosHash(rng.gen()); // EMPTY
        vertex_hashes[1 * VIRT_LEN + col + row * size] = PosHash(rng.gen()); // BLACK
        vertex_hashes[2 * VIRT_LEN + col + row * size] = PosHash(rng.gen()); // WHITE
      }
    }

    return BoardHasher{
      vertex_hashes: vertex_hashes,
    };
  }

  pub fn hash(&self, game: &GoGame) -> PosHash {
    let mut hash = PosHash(0);
    for col in 0 .. game.size {
      for row in 0 .. game.size {
        let v = Vertex::new(row as i16, col as i16);
        hash = hash ^ self.hash_for(v, game.stone_at(v));
      }
    }
    return hash;
  }

  // Calculates zobrist hash for a vertex. Used for super-ko detection.
  fn hash_for(&self, vertex: Vertex, stone: Stone) -> PosHash {
    let offset = match stone {
      stone::EMPTY => 0,
      stone::BLACK => 1,
      stone::WHITE => 2,
      stone::BORDER => 3,
      _ => panic!("unknown stone"),
    };
    return self.vertex_hashes[offset * VIRT_LEN + vertex.as_index()];
  }
}
