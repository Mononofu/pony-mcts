extern crate rand;

use go::GoGame;
use mcts::Controller;

pub struct Engine {
  game: GoGame,
  controller: Controller,
  rng: rand::StdRng,
  pub running: bool,
}

impl Engine {
  pub fn new(rng: rand::StdRng) -> Engine {
    Engine {
      game: GoGame::new(9),
      controller: Controller::new(),
      rng: rng,
      running: true,
    }
  }

  pub fn execute(&mut self, command: String) -> String {
    let command = command.split(" ").next().unwrap().to_string();
    if command == "name" {
      "= rust_mcts".to_string()
    } else if command == "genmove" {
      let m = self.controller.gen_move(&self.game, 20000, &mut self.rng);
      format!("= {}", m)
    } else if command == "gogui-analyze_commands" {
      "= gfx/move_values/move_values".to_string()
    } else if command == "move_values" {
      let m = self.controller.gen_move(&self.game, 50000, &mut self.rng);
      let mut res = "= ".to_string();
      for c in self.controller.root.children.iter() {
        res.push_str(&format!("COLOR #0000{:02x} {}\n",
          (c.uct(20000, 0f64) * 255f64) as u8, c.vertex));
        res.push_str(&format!("LABEL {} {}\n", c.vertex,
          (c.uct(20000, 0f64) * 100f64) as u8));
      }
      res
    } else if command == "boardsize" {
      "=".to_string()
    } else if command == "version" {
      "= 1".to_string()
    }  else if command == "clear_board" {
      "=".to_string()
    } else if command == "list_commands" {
      "= name gogui-analyze_commands move_values boardsize list_commands quit".to_string()
    } else if command == "quit" {
      self.running = false;
      "=".to_string()
    } else {
      "? unknown command".to_string()
    }
  }
}
