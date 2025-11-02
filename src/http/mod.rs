use std::{
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    collections::HashMap,
};

#[derive(Eq, Hash, PartialEq)]
pub enum HTTPRequestMethods {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    NONE,
}

struct HTTPRequest {
    method: HTTPRequestMethods,
    endpoint: String, 
}

pub struct HTTPServer {
    endpoints: HashMap<(String, HTTPRequestMethods), fn() -> String>,
    address: String,
}

impl HTTPServer {
    pub fn new(address: String) -> HTTPServer {
        HTTPServer {
            endpoints: HashMap::new(),
            address: address,
        }
    }
    pub fn add_endpoint(&mut self, enpoint: String, method: HTTPRequestMethods, function: fn() -> String) {
        self.endpoints.insert((enpoint, method), function);
    }
    pub fn listen(&self) {
        let listener = create_tcp_listener(self.address.as_str());

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let buf_reader = BufReader::new(&stream);

            let request_line = buf_reader.lines().next().unwrap().unwrap();
            let http_request = parse_request(request_line.as_str());

            let endpoint = self.endpoints.get(&(http_request.endpoint, http_request.method)).unwrap();

            stream.write_all(endpoint().as_bytes()).unwrap();
        }
    }
}

pub fn create_http_response(status_code: u16, content: &str, content_type: &str) -> String {
    format!("{}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n{}", parse_http_status_code(status_code), content.len(), content_type, content)
}

fn create_tcp_listener(address: &str) -> TcpListener {
    TcpListener::bind(address).unwrap()
}

fn parse_http_method(method_str: &str) -> HTTPRequestMethods {
    match method_str {
        "GET" => HTTPRequestMethods::GET,
        "POST" => HTTPRequestMethods::POST,
        "PUT" => HTTPRequestMethods::PUT,
        "DELETE" => HTTPRequestMethods::DELETE,
        "PATCH" => HTTPRequestMethods::PATCH,
        _ => HTTPRequestMethods::NONE,
    }
}

fn parse_http_status_code(code: u16) -> String {
    match code {
        200 => "HTTP/1.1 200 OK".to_string(),
        404 => "HTTP/1.1 404 Not Found".to_string(),
        _ => "".to_string(),
    }
}

/// Parses the first line of a http header in order to build a HTTPRequest struct
fn parse_request(request_line: &str) -> HTTPRequest {
    let rs: Vec<&str> = request_line.split(" ").collect();

    HTTPRequest {
        method: parse_http_method(rs.get(0).unwrap()),
        endpoint: rs.get(1).unwrap().to_string()
    }
}
