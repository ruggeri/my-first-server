use super::worker_manager::WorkerManager;

type Work = dyn Fn() -> () + Send;

pub struct ThreadPool {
  managers: Vec<WorkerManager>,
}

impl ThreadPool {
  pub fn new(num_threads: u32) -> ThreadPool {
    let mut managers = vec![];

    for _ in 0..num_threads {
      managers.push(WorkerManager::new());
    }

    ThreadPool {
      managers
    }
  }

  pub fn least_busy_manager(&self) -> &WorkerManager {
    let least_busy_idx = (0..self.managers.len()).fold(0, |min_idx, current_idx| {
      let min_tasks = self.managers[min_idx].work_count();
      let num_tasks = self.managers[current_idx].work_count();

      if num_tasks < min_tasks {
        current_idx
      } else {
        min_idx
      }
    });

    &self.managers[least_busy_idx]
  }

  pub fn send_work(&self, work: Box<Work>) {
    self.least_busy_manager().send_work(work);
  }

  pub fn run(&mut self) {
    self.managers.iter_mut().for_each(|m| { m.run(); });
  }

  pub fn shut_down(&self) {
    self.managers.iter().for_each(|m| { m.shut_down(); });
  }
}
