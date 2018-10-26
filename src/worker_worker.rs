use crossbeam_channel as channel;

type Work = dyn Fn() -> () + Send;
// Kind of annoying that I can't use FnOnce, because it plays really
// badly with Box. There is an unstable FnBox trait for this purpose...
type WorkCompletionHandler = dyn Fn() -> () + Send;

pub enum WorkMessage {
  ShutDown,
  Assignment(Box<Work>),
}

pub struct WorkerWorker {
  rx: channel::Receiver<WorkMessage>,
  work_completion_handler: Option<Box<WorkCompletionHandler>>,
}

impl WorkerWorker {
  pub fn new(rx: channel::Receiver<WorkMessage>) -> WorkerWorker {
    WorkerWorker {
      rx,
      work_completion_handler: None,
    }
  }

  pub fn set_work_completion_handler(&mut self, work_completion_handler: Box<WorkCompletionHandler>) {
    self.work_completion_handler = Some(work_completion_handler);
  }

  fn perform_work(&self, work: Box<Work>) {
    (*work)();

    if let Some(ref work_completion_handler) = self.work_completion_handler {
      (*work_completion_handler)();
    }
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
}
