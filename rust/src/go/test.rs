use super::GoGame;
use super::Stone;
use super::NEIGHBOURS;
use super::DIAG_NEIGHBOURS;
use super::VIRT_LEN;

extern crate rand;
use rand::SeedableRng;

#[test]
fn stone_opponent() {
  assert_eq!(Stone::White, Stone::Black.opponent());
  assert_eq!(Stone::Black, Stone::White.opponent());
}

#[test]
fn vertex_neighbours() {
  for col in 0 .. 19 {
    for row in 0 .. 19 {
      let mut expected = vec![GoGame::vertex(col - 1, row),
                              GoGame::vertex(col + 1, row),
                              GoGame::vertex(col, row - 1),
                              GoGame::vertex(col, row + 1)].clone();
      expected.sort();
      let mut got = NEIGHBOURS[GoGame::vertex(col, row).as_index()].clone();
      got.sort();
      assert_eq!(expected, got);

      let mut expected = vec![GoGame::vertex(col - 1, row - 1),
                              GoGame::vertex(col + 1, row - 1),
                              GoGame::vertex(col - 1, row + 1),
                              GoGame::vertex(col + 1, row + 1)].clone();
      expected.sort();
      let mut got = DIAG_NEIGHBOURS[GoGame::vertex(col, row).as_index()].clone();
      got.sort();
      assert_eq!(expected, got);
    }
  }
}

#[test]
fn can_play_single_stone() {
  let mut game = GoGame::new(9);
  let v = GoGame::vertex(2, 2);
  game.play(Stone::Black, v);
  assert_eq!(4, game.num_pseudo_liberties(v));
  assert_eq!(false, game.can_play(Stone::Black, v));
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

#[test]
fn uniform_move_distribution() {
  let mut rng = rand::StdRng::from_seed(&[42]);
  let mut game = GoGame::new(9);
  let num_valid_moves = game.possible_moves(Stone::Black).len() as f64;
  let num_samples = 100000;
  let mut count = vec![0; VIRT_LEN];
  for _ in 0 .. num_samples {
    count[game.random_move(Stone::Black, &mut rng).as_index()] += 1;
  }
  for v in game.possible_moves(Stone::Black) {
    let frac = count[v.as_index()] as f64 / num_samples as f64 * num_valid_moves;
    assert!(frac > 0.9 && frac < 1.1, format!("{}", frac));
  }
}
