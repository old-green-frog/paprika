#[allow(dead_code)]
#[allow(unused_imports)]
use std::net::TcpListener;

mod helpers;

#[allow(dead_code)]
pub struct Server<T: Response + 'static + Sized> {
    address: String,
    listener: TcpListener,
    handlers: Vec<(
        &'static (dyn Fn() -> T + Send + Sync + 'static),
        &'static [&'static str],
        &'static str,
    )>,
    streams_count: usize,
}

impl<T> Server<T>
where
    T: Response + Send,
{
    pub fn from_address(addr: &str) -> Server<T> {
        Server {
            address: String::from(addr),
            listener: TcpListener::bind(addr).unwrap(),
            handlers: Vec::new(),
            streams_count: 10
        }
    }

    #[allow(dead_code)]
    pub fn handle<'a>(
        &mut self,
        methods: &'static [&'a str],
        route: &'a str,
        func: &'static (impl Fn() -> T + Send + Sync),
    ) {
        self.handlers.push((func, methods, route));
    }

    pub fn run(&self) {
        let pull = helpers::ThreadPool::new(self.streams_count);

        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            let handlers = self.handlers.clone();

            for handler in handlers {
                let stream = stream.try_clone().unwrap();
                pull.execute(move || helpers::handle_connection(&stream, handler));
            }
        }
    }

    pub fn run_from_threads(&mut self, threads: usize) {
        self.streams_count = threads;
        let pull = helpers::ThreadPool::new(self.streams_count);

        for stream in self.listener.incoming() {
            let stream = stream.unwrap();
            let handlers = self.handlers.clone();

            for handler in handlers {
                let stream = stream.try_clone().unwrap();
                pull.execute(move || helpers::handle_connection(&stream, handler));
            }
        }
    }
}

pub trait Response {
    fn as_any(&self) -> &dyn std::any::Any;
}

#[derive(Debug)]
pub enum HttpResponse {
    Ok,
    NotFound,
    Content([String; 2]),
}

impl Response for HttpResponse {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl HttpResponse {
    pub fn from_text(&self, content: &str) -> Self {
        match self {
            HttpResponse::Ok => {
                HttpResponse::Content([String::from("HTTP/1.1 200 OK"), String::from(content)])
            },
            HttpResponse::NotFound => {
                HttpResponse::Content([String::from("HTTP/1.01 404 NOT FOUND"), String::from(content)])
            }
            _ => {
                panic!("Error when parse content!");
            }
        }
    }

    pub fn default(&self) -> Self {
        match self {
            HttpResponse::Ok => HttpResponse::Content([String::from("HTTP/1.1 200 OK"), String::from("Hello World!")]),
            HttpResponse::NotFound => HttpResponse::Content([String::from("HTTP/1.01 404 NOT FOUND"), String::from("Not found!")]),
            HttpResponse::Content(text) => HttpResponse::Content([String::from("HTTP/1.1 200 OK"), String::from(&text[0])]),
        }
    }
}
