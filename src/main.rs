mod go;

fn main() {
  let mut game = go::GoGame::new(9);
  game.play(go::Stone::Black, go::Vertex{x: 4, y: 4});
  game.play(go::Stone::White, go::Vertex{x: 5, y: 4});
  game.play(go::Stone::White, go::Vertex{x: 3, y: 4});
  game.play(go::Stone::White, go::Vertex{x: 4, y: 3});
  game.play(go::Stone::White, go::Vertex{x: 4, y: 5});
  println!("{}", game);
}
