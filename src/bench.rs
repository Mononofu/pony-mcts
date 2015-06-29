extern crate time;

use std::collections;

#[derive(Clone)]
pub struct Timer {
  time_spent: collections::HashMap<String, time::Duration>,
}

pub struct FunctionScope<'a> {
  parent: &'a mut Timer,
  name: String,
  start: time::PreciseTime,
}

impl<'a> Drop for FunctionScope<'a> {
  fn drop(&mut self) {
    self.parent.update_time(self.name.clone(), self.start.to(time::PreciseTime::now()));
  }
}

impl Timer {
  pub fn new() -> Timer {
    Timer{
      time_spent: collections::HashMap::new(),
    }
  }

  pub fn func<'a>(&'a mut self, name: &str) -> FunctionScope<'a> {
    FunctionScope{
      parent: self,
      name: name.to_string(),
      start: time::PreciseTime::now(),
    }
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

impl Drop for Timer {
  fn drop(&mut self) {
    println!("execution trace:");
    for (name, &duration) in self.time_spent.iter() {
      println!("{}: {}", name, duration);
    }
  }
}
