#[allow(dead_code)]
#[allow(unused_imports)]
use std::net::TcpStream;

#[allow(unused_imports)]
use std::ops::FnOnce;

#[allow(unused_imports)]
use std::thread;
#[allow(unused_imports)]
use std::time::Duration;

#[allow(unused_imports)]
use std::sync::mpsc;

#[allow(unused_imports)]
use std::sync::{Arc, Mutex};

use std::io::prelude::*;

#[allow(dead_code)]
struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = reciever.lock().unwrap().recv().unwrap();
            job();
        });

        Worker { id, thread }
    }
}

#[allow(dead_code)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, reciever) = mpsc::channel();
        let mut workers = Vec::with_capacity(size);

        let reciever = Arc::new(Mutex::new(reciever));

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&reciever)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

#[allow(unused_parens)]
pub fn handle_connection<'a, T>(
    mut stream: &TcpStream,
    contents: (&'a (dyn Fn() -> T + Send + Sync), &'a [&'a str], &'a str),
) where
    T: crate::Response,
{
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let (handler, methods, route) = contents;

    for method in methods {
        let get_str = format!("{} {} HTTP/1.1\r\n", method, route);

        let [status_line, contents] = if buffer.starts_with(get_str.as_bytes()) {
            release_handler(handler)
        } else {
            [String::from("HTTP/1.01 404 NOT FOUND"), String::from("Not found!")]
        };

        let response = format!(
            "{}\r\nContent-Length: {}\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n{}",
            status_line,
            contents.len(),
            contents
        );

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}

fn release_handler<F, T>(func: F) -> [String; 2]
where
    F: Fn() -> T,
    T: crate::Response,
{
    let handler = func();
    let handler = handler.as_any()
                         .downcast_ref::<crate::HttpResponse>()
                         .unwrap();

    if let crate::HttpResponse::Content(text) = handler {
        text.clone()
    } else {
        panic!("Error when parsing handler contents")
    }
}
