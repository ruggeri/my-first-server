use crossbeam_channel as channel;
use std::sync::{Arc, Mutex};
use std::thread;
use super::worker_worker::{WorkMessage, WorkerWorker};

type Work = dyn Fn() -> () + Send;
type WorkCountMutex = Arc<Mutex<usize>>;

pub struct WorkerManager {
  tx: channel::Sender<WorkMessage>,
  // When not running the worker, store it.
  worker: Arc<WorkerWorker>,
  // When running the worker, store the handle that will give it back to
  // us.
  work_count: WorkCountMutex,
}

impl WorkerManager {
  pub fn new() -> WorkerManager {
    let (tx, rx) = channel::unbounded::<WorkMessage>();
    let work_count = Arc::new(Mutex::new(0));

    let worker = WorkerManager::new_worker_worker(rx, Arc::clone(&work_count));

    let manager = WorkerManager {
      tx,
      worker: Arc::new(worker),
      work_count,
    };

    manager
  }

  // Builds a WorkerWorker to manage.
  fn new_worker_worker(rx: channel::Receiver<WorkMessage>, work_count: Arc<Mutex<usize>>) -> WorkerWorker {
    // The completion handler will let the Manager know how to maintain
    // work count.
    let work_completion_handler = move || {
      let mut work_count = (*work_count).lock().unwrap();
      *work_count -= 1;
    };

    WorkerWorker::new(rx, Box::new(work_completion_handler))
  }

  pub fn send_work(&self, work: Box<Work>) {
    let mut work_count = (*self.work_count).lock().unwrap();
    *work_count += 1;

    self.tx.send(WorkMessage::Assignment(work));
  }

  // Stop the worker. Will be able to resume if you want.
  pub fn shut_down(&mut self) {
    if !self.worker.is_running() {
      panic!("Trying to shut down worker that is not running.");
    }

    self.tx.send(WorkMessage::ShutDown);
    self.worker.join()
  }

  pub fn run(&mut self) {
    if self.worker.is_running() {
      panic!("Trying to run worker that is already running.");
    }

    // Fork a thread and save the join handle.
    // TODO: This is still awkward.
    let worker = Arc::clone(&self.worker);
    let join_handle = thread::spawn(move || {
      worker.run();
    });

    let mut join_handle_option = self.worker.join_handle();
    *join_handle_option = Some(join_handle);
  }

  pub fn work_count(&self) -> usize {
    *(*self.work_count).lock().unwrap()
  }
}
