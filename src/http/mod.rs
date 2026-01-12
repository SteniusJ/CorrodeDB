use std::{
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    collections::HashMap,
};

#[derive(Eq, Hash, PartialEq, Debug)]
pub enum HTTPRequestMethods {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    NONE,
}

pub struct HTTPServer<'a> {
    endpoints: HashMap<(String, HTTPRequestMethods), Box<dyn FnMut(String) -> String + 'a>>,
    address: String,
}

#[derive(Debug)]
struct HTTPRequestHeader {
    method: HTTPRequestMethods,
    endpoint: String,
    content_type: String,
    content_length: u64,
    content: String,
}

impl<'a> HTTPServer<'a> {
    pub fn new(address: String) -> HTTPServer<'a> {
        HTTPServer {
            endpoints: HashMap::new(),
            address: address,
        }
    }
    pub fn add_endpoint(&mut self, enpoint: &str, method: HTTPRequestMethods, closure: impl FnMut(String) -> String + 'a) {
        self.endpoints.insert((enpoint.to_string(), method), Box::new(closure));
    }
    pub fn listen(&mut self) {
        let listener = create_tcp_listener(self.address.as_str());
        
        println!("http server listening on: {}", self.address.as_str());

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let buf_reader = BufReader::new(&stream);

            let request_header = parse_http_request_header(buf_reader.lines().into_iter());

            println!("{request_header:?}");

            let endpoint = match self.endpoints.get_mut(&(request_header.endpoint, request_header.method)) {
                Some(ep) => ep(request_header.content),
                None => create_http_response(404, "text/html; charset=utf-8", "Endpoint not found"),
            };

            stream.write_all(endpoint.as_bytes()).unwrap();
        }
    }
}

pub fn create_http_response(status_code: u16, content_type: &str, content: &str) -> String {
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

fn parse_http_request_header(header_iter: std::io::Lines<BufReader<&TcpStream>>) -> HTTPRequestHeader {
    let mut request_header = HTTPRequestHeader {
        method: HTTPRequestMethods::NONE,
        endpoint: String::new(),
        content_type: String::new(),
        content_length: 0,
        content: String::new(),
    };

    let mut building_content = false;
    let mut content_length: u64 = 0;
    for line_result in header_iter.enumerate() {
        let line = line_result.1.unwrap();
        let line_split: Vec<&str> = line.split(": ").collect();

        if building_content {
            content_length += line.len() as u64 + 2; // adding 2 for the newline characters "\n"
            request_header.content += format!("{}", line).as_str();

            if content_length == request_header.content_length {
                return request_header;
            }
        }

        if line_result.0 == 0 {
            let line_split: Vec<&str> = line.split_whitespace().collect();

            request_header.method = parse_http_method(line_split.get(0).unwrap());
            request_header.endpoint = line_split.get(1).unwrap().to_string();
            continue;
        }

        match *line_split.get(0).unwrap() {
            "Content-Type" => request_header.content_type = line_split.get(1).unwrap().to_string(),
            "Content-Length" => request_header.content_length = line_split.get(1).unwrap().parse().unwrap(),
            "" => building_content = true,
            _ => (),
        }
    }

    request_header
}
