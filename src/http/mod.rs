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

/// Parses the first line of a http header in order to build a HTTPRequest struct
fn parse_request(request_line: &str) -> HTTPRequest {
    let rs: Vec<&str> = request_line.split(" ").collect();

    HTTPRequest {
        method: parse_http_method(rs.get(0).unwrap()),
        endpoint: rs.get(1).unwrap().to_string()
    }
}

pub fn listen(address: &str, endpoints: HashMap<(String, HTTPRequestMethods), fn() -> String>) {
    let listener = create_tcp_listener(address);

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let buf_reader = BufReader::new(&stream);

        let request_line = buf_reader.lines().next().unwrap().unwrap();
        let http_request = parse_request(request_line.as_str());

        let endpoint = endpoints.get(&(http_request.endpoint, http_request.method)).unwrap();

        stream.write_all(endpoint().as_bytes()).unwrap();
    }
}
