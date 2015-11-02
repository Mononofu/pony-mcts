use std::collections;

use super::zobrist::BoardHasher;
use super::zobrist::PosHash;
use super::super::go::GoGame;

fn generate_hashes(depth: usize, hasher: &BoardHasher, game: &mut GoGame, 
			seen: &mut collections::HashMap<PosHash, GoGame>) {
	if depth <= 0 {
		return;
	}

	for v in game.possible_moves(game.to_play) {
		let c = game.to_play;
		game.play(c, v);

		let hash = hasher.hash(&game);
		if seen.contains_key(&hash) {
			assert_eq!(game, seen.get(&hash).unwrap());
		}

		seen.insert(hash, game.clone());

		generate_hashes(depth - 1, hasher, game, seen);

		game.undo(1);
	}
}

#[test]
fn hash_collision() {
	let hasher = BoardHasher::new();
	let mut game = GoGame::new(9);
  let mut seen = collections::HashMap::<PosHash, GoGame>::new();

  generate_hashes(2, &hasher, &mut game, &mut seen);
}