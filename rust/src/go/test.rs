use super::GoGame;
use super::Stone;
use super::Vertex;
use super::String;

extern crate rand;
use rand::Rng;
use rand::SeedableRng;

#[test]
fn can_play_single_stone() {
  let mut game = GoGame::new(9);
  let v = GoGame::vertex(2, 2);
  game.play(Stone::Black, v);
  assert_eq!(4, game.num_pseudo_liberties(v));
}

#[test]
fn can_remove_liberties() {
  let mut game = GoGame::new(9);
  game.play(Stone::Black, GoGame::vertex(2, 2));
  game.play(Stone::White, GoGame::vertex(3, 2));
  assert_eq!(3, game.num_pseudo_liberties(GoGame::vertex(2, 2)));
  assert_eq!(3, game.num_pseudo_liberties(GoGame::vertex(3, 2)));
}

#[test]
fn can_join_strings() {
  let mut game = GoGame::new(9);
  let v = GoGame::vertex(2, 2);
  game.play(Stone::Black, GoGame::vertex(2, 2));
  game.play(Stone::Black, GoGame::vertex(3, 2));
  assert_eq!(6, game.num_pseudo_liberties(v));
}

#[test]
fn can_capture_single_stone() {
  let mut game = GoGame::new(9);
  game.play(Stone::White, GoGame::vertex(2, 2));
  game.play(Stone::Black, GoGame::vertex(1, 2));
  game.play(Stone::Black, GoGame::vertex(3, 2));
  game.play(Stone::Black, GoGame::vertex(2, 1));
  game.play(Stone::Black, GoGame::vertex(2, 3));
  assert_eq!(Stone::Empty, game.stone_at(GoGame::vertex(2, 2)));
}

#[test]
fn freedoms_after_capture() {
  let mut game = GoGame::new(9);
  game.play(Stone::White, GoGame::vertex(0, 0));
  game.play(Stone::Black, GoGame::vertex(1, 0));
  game.play(Stone::Black, GoGame::vertex(1, 1));
  game.play(Stone::Black, GoGame::vertex(0, 1));
  assert_eq!(Stone::Empty, game.stone_at(GoGame::vertex(0, 0)));
  assert_eq!(6, game.num_pseudo_liberties(GoGame::vertex(0, 1)));
}

#[test]
fn initially_all_moves_possible() {
  let mut game = GoGame::new(9);
  assert_eq!(game.possible_moves(Stone::Black).len(), 81);
}

#[test]
fn forbid_filling_real_eye() {
  let mut game = GoGame::new(9);
  game.play(Stone::Black, GoGame::vertex(0, 0));
  game.play(Stone::Black, GoGame::vertex(0, 1));
  game.play(Stone::Black, GoGame::vertex(0, 2));
  game.play(Stone::Black, GoGame::vertex(1, 0));
  game.play(Stone::Black, GoGame::vertex(1, 2));
  game.play(Stone::Black, GoGame::vertex(2, 0));
  game.play(Stone::Black, GoGame::vertex(2, 1));
  assert_eq!(false, game.can_play(Stone::Black, GoGame::vertex(1, 1)));
}

#[test]
fn forbid_filling_real_eyes_of_split_group() {
  let mut game = GoGame::new(9);
  game.play(Stone::Black, GoGame::vertex(0, 0));
  game.play(Stone::Black, GoGame::vertex(0, 2));
  game.play(Stone::Black, GoGame::vertex(1, 1));
  game.play(Stone::Black, GoGame::vertex(1, 2));
  game.play(Stone::Black, GoGame::vertex(2, 0));
  game.play(Stone::Black, GoGame::vertex(2, 1));
  assert_eq!(false, game.can_play(Stone::Black, GoGame::vertex(0, 1)));
  assert_eq!(false, game.can_play(Stone::Black, GoGame::vertex(1, 0)));
}

