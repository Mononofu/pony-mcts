extern crate rand;

use go::GoGame;
use mcts::Controller;
use std::collections;
use go::Stone;
use go::Vertex;
extern crate time;

pub struct Engine {
  game: GoGame,
  controller: Controller,
  rng: rand::StdRng,
  commands: collections::HashMap<String, fn(&mut Engine, Vec<&str>) -> Result<String, String> >,
  analyze_commands: collections::HashMap<String, fn(&mut Engine, Vec<&str>) -> Result<String, String> >,
  pub running: bool,
}

impl Engine {
  pub fn new(rng: rand::StdRng) -> Engine {
    let mut analyze_commands: collections::HashMap<String, fn(&mut Engine, Vec<&str>) -> Result<String, String> > = collections::HashMap::new();
    let mut commands: collections::HashMap<String, fn(&mut Engine, Vec<&str>) -> Result<String, String> > = collections::HashMap::new();
    commands.insert("quit".to_string(), Engine::quit);
    commands.insert("name".to_string(), Engine::name);
    commands.insert("boardsize".to_string(), Engine::boardsize);
    commands.insert("version".to_string(), Engine::version);
    commands.insert("clear_board".to_string(), Engine::clear_board);
    commands.insert("list_commands".to_string(), Engine::list_commands);
    commands.insert("genmove".to_string(), Engine::genmove);
    commands.insert("play".to_string(), Engine::play);
    commands.insert("gogui-analyze_commands".to_string(), Engine::gogui_analyze_commands);

    analyze_commands.insert("move_values".to_string(), Engine::move_values);

    Engine {
      game: GoGame::new(9),
      controller: Controller::new(),
      rng: rng,
      commands: commands,
      analyze_commands: analyze_commands,
      running: true,
    }
  }

  pub fn execute(&mut self, command: String) -> String {
    let args = command.split(" ").collect::<Vec<_>>();
    if args.is_empty() {
      return "? must specify a command".to_string();
    }
    let res = if self.commands.contains_key(args[0]) {
      self.commands[args[0]](self, args)
    } else if self.analyze_commands.contains_key(args[0]) {
      self.analyze_commands[args[0]](self, args)
    } else {
      Err("unknown command".to_string())
    };

    match res {
      Ok(msg) => format!("= {}", msg),
      Err(msg) => format!("? {}", msg),
    }
  }

  fn play(&mut self, args: Vec<&str>) -> Result<String, String> {
    if args.len() != 3 {
      return Err("expected: play C V".to_string());
    }
    self.game.play(try!(args[1].parse::<Stone>()), try!(args[2].parse::<Vertex>()));
    info!("new board:\n{:?}", self.game);
    Ok("".to_string())
  }

  fn genmove(&mut self, args: Vec<&str>) -> Result<String, String> {
    if args.len() != 2 {
      return Err("expected: genmove C".to_string());
    }
    let start = time::PreciseTime::now();
    let color = try!(args[1].parse::<Stone>());
    self.game.to_play = color;
    let num_simulations = 10000;
    let v = self.controller.gen_move(&self.game, num_simulations, &mut self.rng);
    self.game.play(color, v);
    let duration = start.to(time::PreciseTime::now());
    info!("generate move in {} ({:.2} kpps)\n{:?}", duration,
      num_simulations as f64 / duration.num_milliseconds() as f64, self.game);
    Ok(format!("{}", v))
  }

  fn move_values(&mut self, _: Vec<&str>) -> Result<String, String> {
    let num_simulations = 100000;
    self.controller.gen_move(&self.game, num_simulations, &mut self.rng);
    let mut res = "".to_string();
    for c in self.controller.root.children.iter() {
      res.push_str(&format!("COLOR #0000{:02x} {}\n",
        (c.uct(num_simulations) * 255f64) as u8, c.vertex));
      res.push_str(&format!("LABEL {} {}\n", c.vertex,
        (c.uct(num_simulations) * 100f64) as u8));
    }
    Ok(res)
  }

  fn quit(&mut self, _: Vec<&str>) -> Result<String, String> {
    self.running = false;
    Ok("".to_string())
  }

  fn version(&mut self, _: Vec<&str>) -> Result<String, String> {
    Ok("1".to_string())
  }

  fn name(&mut self, _: Vec<&str>) -> Result<String, String> {
    Ok("ErGo".to_string())
  }

  fn clear_board(&mut self, _: Vec<&str>) -> Result<String, String> {
    self.game.reset();
    Ok("".to_string())
  }

  fn boardsize(&mut self, args: Vec<&str>) -> Result<String, String> {
    if args.len() != 2 {
      return Err("expected: boardsize N".to_string());
    }
    let n = args[1].parse::<usize>();
    if n.is_err() {
      return Err(format!("expected integer, got '{}'", args[1]));
    }
    self.game = GoGame::new(n.unwrap());
    Ok("".to_string())
  }

  fn list_commands(&mut self, _: Vec<&str>) -> Result<String, String> {
    Ok(self.commands.keys().map(|s| s.clone())
      .collect::<Vec<String>>().connect(" "))
  }

  fn gogui_analyze_commands(&mut self, _: Vec<&str>) -> Result<String, String> {
    Ok(self.analyze_commands.keys().map(|s| format!("gfx/{}/{}", s, s))
      .collect::<Vec<String>>().connect(" "))
  }
}
