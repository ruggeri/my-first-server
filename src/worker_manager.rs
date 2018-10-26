use crossbeam_channel as channel;
use std::mem;
use std::sync::{Arc, Mutex};
use std::thread;
use super::worker_worker::{WorkMessage, WorkerWorker};

type Work = dyn Fn() -> () + Send;
type WorkCountMutex = Arc<Mutex<usize>>;

pub struct WorkerManager {
  tx: channel::Sender<WorkMessage>,
  worker: Option<WorkerWorker>,
  join_handle: Option<thread::JoinHandle<WorkerWorker>>,
  work_count: WorkCountMutex,
}

impl WorkerManager {
  pub fn new() -> WorkerManager {
    let (tx, rx) = channel::unbounded::<WorkMessage>();
    let work_count = Arc::new(Mutex::new(0));
    let work_count_copy = Arc::clone(&work_count);

    let mut worker = WorkerWorker::new(rx);

    let work_completion_handler = move || {
      let mut work_count = (*work_count_copy).lock().unwrap();
      *work_count -= 1;
    };
    worker.set_work_completion_handler(Box::new(work_completion_handler));

    let manager = WorkerManager {
      tx,
      worker: Some(worker),
      join_handle: None,
      work_count,
    };

    manager
  }

  pub fn send_work(&self, work: Box<Work>) {
    let mut work_count = (*self.work_count).lock().unwrap();
    *work_count += 1;

    self.tx.send(WorkMessage::Assignment(work));
  }

  pub fn shut_down(&self) {
    self.tx.send(WorkMessage::ShutDown);
  }

  pub fn run(&mut self) {
    if self.worker.is_none() {
      panic!("Manager tried to run worker, but worker was already running");
    }

    let mut worker_option = None;
    mem::swap(&mut self.worker, &mut worker_option);
    let worker = worker_option.unwrap();

    let join_handle = thread::spawn(move || {
      worker.run();
      worker
    });

    self.join_handle = Some(join_handle);
  }

  pub fn work_count(&self) -> usize {
    *(*self.work_count).lock().unwrap()
  }
}
