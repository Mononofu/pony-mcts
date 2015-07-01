extern crate time;

use std::collections;

#[derive(Clone)]
pub struct FunctionScope {
  name: String,
  start: time::PreciseTime,
  section_name: Option<String>,
  section_start: time::PreciseTime,
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
    let mut names = self.time_spent.keys().map(|s| s.clone()).collect::<Vec<_>>();
    names.sort();
    for name in names.iter() {
      println!("{:25}: {:4} calls in {}", name, self.num_calls[name],
          self.time_spent[name] * 1000);
    }
  }

  pub fn start(&mut self, name: &str) {
    self.active.push(FunctionScope{
      name: name.to_string(),
      start: time::PreciseTime::now(),
      section_name: None,
      section_start: time::PreciseTime::now(),
    });
  }

  pub fn section(&mut self, name: &str) {
    self.section_end();
    match self.active.last_mut() {
      Some(fs) => {
        fs.section_name = Some(name.to_string());
        fs.section_start = time::PreciseTime::now();
      },
      None => (),
    }
  }

  fn section_end(&mut self) {
    if self.active.is_empty() {
      return;
    }
    let fs = self.active.last().unwrap().clone();
    if let Some(n) = fs.section_name {
      let name = format!("{} - {}", fs.name, n);
      *self.num_calls.entry(name.clone()).or_insert(0) += 1;
      self.update_time(name, fs.section_start.to(time::PreciseTime::now()));
    }
  }

  pub fn end(&mut self) {
    self.section_end();
    match self.active.pop() {
      Some(FunctionScope{name: n, start: s, ..}) => {
        *self.num_calls.entry(n.clone()).or_insert(0) += 1;
        self.update_time(n, s.to(time::PreciseTime::now()))
      },
      None => println!("No active function!"),
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
