use crossbeam_channel as channel;
use std::mem;
use std::sync::{Arc, Mutex};
use std::thread;
use super::worker_worker::{WorkMessage, WorkerWorker};

type Work = dyn Fn() -> () + Send;
type WorkCountMutex = Arc<Mutex<usize>>;

pub struct WorkerManager {
  tx: channel::Sender<WorkMessage>,
  // When not running the worker, store it.
  worker: Option<WorkerWorker>,
  // When running the worker, store the handle that will give it back to
  // us.
  join_handle: Option<thread::JoinHandle<WorkerWorker>>,
  work_count: WorkCountMutex,
}

impl WorkerManager {
  pub fn new() -> WorkerManager {
    let (tx, rx) = channel::unbounded::<WorkMessage>();
    let work_count = Arc::new(Mutex::new(0));

    let worker = WorkerManager::new_worker_worker(rx, Arc::clone(&work_count));

    let manager = WorkerManager {
      tx,
      worker: Some(worker),
      join_handle: None,
      work_count,
    };

    manager
  }

  // Builds a WorkerWorker to manage.
  fn new_worker_worker(rx: channel::Receiver<WorkMessage>, work_count: Arc<Mutex<usize>>) -> WorkerWorker {
    let mut worker = WorkerWorker::new(rx);

    // The completion handler will let the Manager know how to maintain
    // work count.
    let work_completion_handler = move || {
      let mut work_count = (*work_count).lock().unwrap();
      *work_count -= 1;
    };
    worker.set_work_completion_handler(Box::new(work_completion_handler));

    worker
  }

  pub fn send_work(&self, work: Box<Work>) {
    let mut work_count = (*self.work_count).lock().unwrap();
    *work_count += 1;

    self.tx.send(WorkMessage::Assignment(work));
  }

  // Stop the worker. Will be able to resume if you want.
  pub fn shut_down(&mut self) {
    if self.join_handle.is_none() {
      panic!("Manager tried to stop worker, but worker was not running.");
    }

    // Ask worker to shut down.
    self.tx.send(WorkMessage::ShutDown);

    // Swap out the join handle for none. Wait for thread to stop.
    let mut join_handle_option = None;
    mem::swap(&mut self.join_handle, &mut join_handle_option);
    let join_handle = join_handle_option.unwrap();
    let worker = join_handle.join().unwrap();

    // Install the worker back in place.
    self.worker = Some(worker);
  }

  pub fn run(&mut self) {
    if self.worker.is_none() {
      panic!("Manager tried to run worker, but worker was already running.");
    }

    // Take away the worker.
    let mut worker_option = None;
    mem::swap(&mut self.worker, &mut worker_option);
    let worker = worker_option.unwrap();

    // Fork a thread and save the join handle.
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
