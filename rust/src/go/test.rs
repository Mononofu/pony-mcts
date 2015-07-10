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
  let mut expected = vec![GoGame::vertex(2, 1), GoGame::vertex(1, 2), GoGame::vertex(3, 2), GoGame::vertex(2, 3)];
  expected.sort();
  let mut got = game.liberties(v).clone();
  got.sort();
  assert_eq!(expected, got);
}

#[test]
fn can_remove_liberties() {
  let mut game = GoGame::new(9);
  game.play(Stone::Black, GoGame::vertex(2, 2));
  game.play(Stone::White, GoGame::vertex(3, 2));
  let mut expected = vec![GoGame::vertex(2, 1), GoGame::vertex(1, 2), GoGame::vertex(2, 3)];
  expected.sort();
  let mut got = game.liberties(GoGame::vertex(2, 2)).clone();
  got.sort();
  assert_eq!(expected, got);

  let mut expected = vec![GoGame::vertex(3, 1), GoGame::vertex(3, 3), GoGame::vertex(4, 2)];
  expected.sort();
  let mut got = game.liberties(GoGame::vertex(3, 2)).clone();
  got.sort();
  assert_eq!(expected, got);
}

#[test]
fn can_join_strings() {
  let mut game = GoGame::new(9);
  let v = GoGame::vertex(2, 2);
  game.play(Stone::Black, GoGame::vertex(2, 2));
  game.play(Stone::Black, GoGame::vertex(3, 2));
  let mut expected = vec![GoGame::vertex(2, 1), GoGame::vertex(1, 2), GoGame::vertex(2, 3),
      GoGame::vertex(3, 1), GoGame::vertex(3, 3), GoGame::vertex(4, 2)];
  expected.sort();
  let mut got = game.liberties(v).clone();
  got.sort();
  assert_eq!(expected, got);
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

  let mut expected = vec![GoGame::vertex(0, 0), GoGame::vertex(0, 2),
      GoGame::vertex(1, 2), GoGame::vertex(2, 0), GoGame::vertex(2, 1)];
  expected.sort();
  let mut got = game.liberties(GoGame::vertex(0, 1)).clone();
  got.sort();
  assert_eq!(expected, got);
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
fn possible_move_caching() {
  let mut game = GoGame::new(9);
  let mut rng = rand::StdRng::from_seed(&[42]);
  let mut color_to_play = Stone::White;
  let mut num_consecutive_passes = 0;
  let mut num_moves = 0;

  while num_consecutive_passes < 2 {
    color_to_play = color_to_play.opponent();
    num_moves += 1;

    let mut possible = game.possible_moves(color_to_play).clone();
    possible.sort();
    let mut cached = game.legal_moves.clone();
    cached.sort();
    println!("{:?}", num_moves);
    println!("{}", game);
    assert_eq!(possible, cached);

    if let Some(v) = rng.choose(&possible) {
      println!("{}", v);
      num_consecutive_passes = 0;
      assert!(game.play(color_to_play, *v));
    } else {
      num_consecutive_passes += 1;
    }
  }


}
