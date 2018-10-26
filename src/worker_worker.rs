use crossbeam_channel as channel;
use std::mem;
use std::sync::{Mutex, MutexGuard};
use std::thread;

type Work = dyn Fn() -> () + Send;
// Kind of annoying that I can't use FnOnce, because it plays really
// badly with Box. There is an unstable FnBox trait for this purpose...
type WorkCompletionHandler = dyn Fn() -> () + Sync + Send;
type WorkerJoinHandle = thread::JoinHandle<()>;

pub enum WorkMessage {
  ShutDown,
  Assignment(Box<Work>),
}

pub struct WorkerWorker {
  rx: channel::Receiver<WorkMessage>,
  work_completion_handler: Box<WorkCompletionHandler>,
  join_handle: Mutex<Option<WorkerJoinHandle>>,
}

impl WorkerWorker {
  pub fn new(rx: channel::Receiver<WorkMessage>, work_completion_handler: Box<WorkCompletionHandler>) -> WorkerWorker {
    WorkerWorker {
      rx,
      work_completion_handler,
      join_handle: Mutex::new(None),
    }
  }

  fn perform_work(&self, work: Box<Work>) {
    (*work)();
    (*self.work_completion_handler)();
  }

  fn recv_work_msg(&self) -> WorkMessage {
    self.rx.recv().expect("No one should close send half of channel!")
  }

  pub fn run(&self) {
    loop {
      match self.recv_work_msg() {
        WorkMessage::ShutDown => break,
        WorkMessage::Assignment(work) => {
          self.perform_work(work);
        },
      }
    }
  }

  pub fn join_handle(&self) -> MutexGuard<Option<WorkerJoinHandle>> {
    self.join_handle.lock().expect("lock holder shouldn't panic")
  }

  pub fn is_running(&self) -> bool {
    let join_handle_option = self.join_handle();
    join_handle_option.is_some()
  }

  pub fn join(&self) {
    let mut join_handle_option = self.join_handle();

    let join_handle_option = mem::replace(
      &mut (*join_handle_option),
      None
    );

    let join_handle = join_handle_option.unwrap();
    join_handle.join().expect("should not panic before join");
  }
}
