extern crate time;

use std::collections;

#[derive(Clone)]
pub struct FunctionScope {
  name: String,
  start: time::PreciseTime,
}

#[derive(Clone)]
pub struct Timer {
  num_calls: collections::HashMap<String, u64>,
  time_spent: collections::HashMap<String, time::Duration>,
  active: Vec<FunctionScope>,
}


impl Timer {
  pub fn new() -> Timer {
    Timer{
      num_calls: collections::HashMap::new(),
      time_spent: collections::HashMap::new(),
      active: Vec::new(),
    }
  }

  pub fn report(&self) {
    for (name, &duration) in self.time_spent.iter() {
      println!("{:20}: {:4} calls in {}", name, self.num_calls[name], duration);
    }
  }

  pub fn start(&mut self, name: &str) {
    // self.active.push(FunctionScope{
    //   name: name.to_string(),
    //   start: time::PreciseTime::now(),
    // });
  }

  pub fn end(&mut self) {
    // match self.active.pop() {
    //   Some(FunctionScope{name: n, start: s}) => {
    //     *self.num_calls.entry(n.clone()).or_insert(0) += 1;
    //     self.update_time(n, s.to(time::PreciseTime::now()))
    //   },
    //   None => println!("No active function!"),
    // }
  }

  fn update_time(&mut self, name: String, d: time::Duration) {
    let old = if self.time_spent.contains_key(&name) {
      self.time_spent[&name]
    } else {
      time::Duration::zero()
    };
    self.time_spent.insert(name, old + d);
  }
}
