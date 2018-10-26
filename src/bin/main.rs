extern crate my_first_server;

use my_first_server::ThreadPool;
use std::io;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

fn handle_connection(read_stream: &TcpStream) -> io::Result<()> {
  let write_stream = read_stream.try_clone()?;
  let reader = io::BufReader::new(read_stream);
  let mut writer = io::BufWriter::new(write_stream);

  for line in reader.lines() {
    let line = line? + "\n";
    let bytes_written = writer.write(line.as_bytes())?;
    if bytes_written != line.len() {
      panic!("Couldn't write everything?");
    }
    writer.flush()?;
  }

  Ok(())
}

fn main() -> io::Result<()> {
  let listener = TcpListener::bind("127.0.0.1:8080")?;
  let mut thread_pool = ThreadPool::new(2);
  thread_pool.run();

  for stream in listener.incoming() {
    let stream = stream?;

    let work = move || {
      if let Err(e) = handle_connection(&stream) {
        println!("Error: {}", e);
      }
    };

    thread_pool.send_work(Box::new(work));
  }

  Ok(())
}
