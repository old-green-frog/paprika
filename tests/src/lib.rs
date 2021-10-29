extern crate minreq;

use paprika::{Server, HttpResponse};


// Simple handler
#[allow(dead_code)]
fn hello() -> HttpResponse {
    HttpResponse::Ok.from_text("Hello World!")
}

#[allow(dead_code)]
fn run_server() {
    let mut server: Server<HttpResponse> = Server::from_address("127.0.0.1:8080");

    server.handle(&["GET"], "/", &hello);
    server.run()
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[ignore]
    fn runner() {
        run_server()
    }


    #[test]
    fn get_hello() {
        let result = minreq::get("http://127.0.0.1:8080").send().unwrap();
        let result = result.as_str().unwrap();

        assert_eq!(result, "Hello World!")
    }
}
