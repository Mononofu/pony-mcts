use super::GoGame;
use super::Stone;
use super::Vertex;
use super::String;

#[test]
fn can_play_single_stone() {
  let mut game = GoGame::new(9);
  let v = Vertex{x: 2, y: 2};
  game.play(Stone::Black, v, false);
  let mut expected = vec![Vertex{x:2, y:1}, Vertex{x:1, y:2}, Vertex{x:3, y:2}, Vertex{x:2, y:3}];
  expected.sort();
  let mut got = game.liberties(v).clone();
  got.sort();
  assert_eq!(expected, got);
}

#[test]
fn can_remove_liberties() {
  let mut game = GoGame::new(9);
  game.play(Stone::Black, Vertex{x: 2, y: 2}, false);
  game.play(Stone::White, Vertex{x: 3, y: 2}, false);
  let mut expected = vec![Vertex{x:2, y:1}, Vertex{x:1, y:2}, Vertex{x:2, y:3}];
  expected.sort();
  let mut got = game.liberties(Vertex{x: 2, y: 2}).clone();
  got.sort();
  assert_eq!(expected, got);

  let mut expected = vec![Vertex{x:3, y:1}, Vertex{x:3, y:3}, Vertex{x:4, y:2}];
  expected.sort();
  let mut got = game.liberties(Vertex{x: 3, y: 2}).clone();
  got.sort();
  assert_eq!(expected, got);
}

#[test]
fn can_join_strings() {
  let mut game = GoGame::new(9);
  let v = Vertex{x: 2, y: 2};
  game.play(Stone::Black, Vertex{x: 2, y: 2}, false);
  game.play(Stone::Black, Vertex{x: 3, y: 2}, false);
  let mut expected = vec![Vertex{x:2, y:1}, Vertex{x:1, y:2}, Vertex{x:2, y:3},
      Vertex{x:3, y:1}, Vertex{x:3, y:3}, Vertex{x:4, y:2}];
  expected.sort();
  let mut got = game.liberties(v).clone();
  got.sort();
  assert_eq!(expected, got);
}

#[test]
fn can_capture_single_stone() {
  let mut game = GoGame::new(9);
  game.play(Stone::White, Vertex{x: 2, y: 2}, false);
  game.play(Stone::Black, Vertex{x: 1, y: 2}, false);
  game.play(Stone::Black, Vertex{x: 3, y: 2}, false);
  game.play(Stone::Black, Vertex{x: 2, y: 1}, false);
  game.play(Stone::Black, Vertex{x: 2, y: 3}, false);
  assert_eq!(None, game.stone_at(Vertex{x: 2, y: 2}));
}

#[test]
fn freedoms_after_capture() {
  let mut game = GoGame::new(9);
  game.play(Stone::White, Vertex{x: 0, y: 0}, false);
  game.play(Stone::Black, Vertex{x: 1, y: 0}, false);
  game.play(Stone::Black, Vertex{x: 1, y: 1}, false);
  game.play(Stone::Black, Vertex{x: 0, y: 1}, false);
  assert_eq!(None, game.stone_at(Vertex{x: 0, y: 0}));

  let mut expected = vec![Vertex{x:0, y:0}, Vertex{x:0, y:2},
      Vertex{x:1, y:2}, Vertex{x:2, y:0}, Vertex{x:2, y:1}];
  assert_eq!(expected, game.liberties(Vertex{x: 0, y: 1}).clone());
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
  a.strings.entry(0).or_insert_with(|| panic!()).stones.push(Vertex{x:0, y:0});
  assert_eq!(0, c.strings[&0].stones.len());
}

