use super::GoGame;
use super::Stone;
use super::Vertex;
use super::String;

#[test]
fn can_play_single_stone() {
  let mut game = GoGame::new(9);
  let v = GoGame::vertex(2, 2);
  game.play(Stone::Black, v, false);
  let mut expected = vec![GoGame::vertex(2, 1), GoGame::vertex(1, 2), GoGame::vertex(3, 2), GoGame::vertex(2, 3)];
  expected.sort();
  let mut got = game.liberties(v).clone();
  got.sort();
  assert_eq!(expected, got);
}

#[test]
fn can_remove_liberties() {
  let mut game = GoGame::new(9);
  game.play(Stone::Black, GoGame::vertex(2, 2), false);
  game.play(Stone::White, GoGame::vertex(3, 2), false);
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
  game.play(Stone::Black, GoGame::vertex(2, 2), false);
  game.play(Stone::Black, GoGame::vertex(3, 2), false);
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
  game.play(Stone::White, GoGame::vertex(2, 2), false);
  game.play(Stone::Black, GoGame::vertex(1, 2), false);
  game.play(Stone::Black, GoGame::vertex(3, 2), false);
  game.play(Stone::Black, GoGame::vertex(2, 1), false);
  game.play(Stone::Black, GoGame::vertex(2, 3), false);
  assert_eq!(None, game.stone_at(GoGame::vertex(2, 2)));
}

#[test]
fn freedoms_after_capture() {
  let mut game = GoGame::new(9);
  game.play(Stone::White, GoGame::vertex(0, 0), false);
  game.play(Stone::Black, GoGame::vertex(1, 0), false);
  game.play(Stone::Black, GoGame::vertex(1, 1), false);
  game.play(Stone::Black, GoGame::vertex(0, 1), false);
  assert_eq!(None, game.stone_at(GoGame::vertex(0, 0)));

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
fn clone_test() {
  let mut a = GoGame::new(19);
  let b = a.clone();
  a.strings.insert(0, String{
    color: Stone::Black,
    stones: vec![],
    liberties: vec![],
  });
  assert_eq!(0, b.strings.len());
  let c = a.clone();
  a.strings.entry(0).or_insert_with(|| panic!()).stones.push(GoGame::vertex(0, 0));
  assert_eq!(0, c.strings[&0].stones.len());
}

