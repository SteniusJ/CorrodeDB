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
    endpoints: HashMap<(String, HTTPRequestMethods), Box<dyn FnMut(String, HashMap<String, String>) -> String + 'a>>,
    middleware: Vec<Box<dyn FnMut(&mut String, &mut HashMap<String, String>) -> (bool, String) + 'a>>,
    address: String,
}

#[derive(Debug)]
struct HTTPRequestHeader {
    method: HTTPRequestMethods,
    endpoint: String,
    content_type: String,
    content_length: u64,
    content: String,
    url_parameters: HashMap<String, String>,
}

impl<'a> HTTPServer<'a> {
    pub fn new(address: String) -> HTTPServer<'a> {
        HTTPServer {
            endpoints: HashMap::new(),
            middleware: Vec::new(),
            address: address,
        }
    }
    pub fn add_endpoint(&mut self, enpoint: &str, method: HTTPRequestMethods, closure: impl FnMut(String, HashMap<String, String>) -> String + 'a) {
        self.endpoints.insert((enpoint.to_string(), method), Box::new(closure));
    }
    pub fn add_middleware(&mut self, closure: impl FnMut(&mut String, &mut HashMap<String, String>) -> (bool, String) + 'a) {
        self.middleware.push(Box::new(closure));
    }
    pub fn listen(&mut self) {
        let listener = create_tcp_listener(self.address.as_str());
        
        println!("http server listening on: {}", self.address.as_str());

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut buf_reader = BufReader::new(&mut stream);

            let mut request_header = parse_http_request_header(&mut buf_reader);

            println!("{request_header:?}");

            let mut res = String::new();

            for middleware in &mut self.middleware {
                let middleware_res = middleware(&mut request_header.content, &mut request_header.url_parameters);
                if !middleware_res.0 {
                    res = create_http_response(401, "application/json", format!("\"error\":\"{}\"", middleware_res.1).as_str());
                    break;
                }
            }

            if res.is_empty() {
                res = match self.endpoints.get_mut(&(request_header.endpoint, request_header.method)) {
                    Some(ep) => ep(request_header.content, request_header.url_parameters),
                    None => create_http_response(404, "text/html; charset=utf-8", "Endpoint not found"),
                };
            }

            stream.write_all(res.as_bytes()).unwrap();
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
        201 => "HTTP/1.1 201 Created".to_string(),
        400 => "HTTP/1.1 400 Bad Request".to_string(),
        401 => "HTTP/1.1 401 Unauthorized".to_string(),
        404 => "HTTP/1.1 404 Not Found".to_string(),
        500 => "HTTP/1.1 500 Internal Server Error".to_string(),
        _ => "".to_string(),
    }
}

fn parse_http_request_header(buf_reader: &mut BufReader<&mut TcpStream>) -> HTTPRequestHeader {
    let mut request_header = HTTPRequestHeader {
        method: HTTPRequestMethods::NONE,
        endpoint: String::new(),
        content_type: String::new(),
        content_length: 0,
        content: String::new(),
        url_parameters: HashMap::new(),
    };

    loop {
        let mut buffer = String::new();
        let bytes = buf_reader.read_line(&mut buffer).unwrap();

        if bytes == 0 || buffer.trim().is_empty() {
            let mut body = vec![0; request_header.content_length as usize];
            if request_header.content_length > 0 {
                buf_reader.read_exact(&mut body).unwrap();
            }
            request_header.content = String::from_utf8_lossy(&body).to_string();
            break;
        }

        let line_split: Vec<&str> = buffer.trim_end().split(": ").collect();

        if line_split.len() == 1 {
            let line_split: Vec<&str> = buffer.split_whitespace().collect();

            request_header.method = parse_http_method(line_split.get(0).unwrap());

            let url_contents: Vec<&str> = line_split.get(1).unwrap().split("?").collect();
            request_header.endpoint = url_contents[0].to_string();

            if url_contents.len() > 1 {
                let mut url_param_split = url_contents[1].split("&");
                loop {
                    let mut url_param_split = match url_param_split.next() {
                        Some(v) => v,
                        None => break,
                    }.split("=");

                    let name = match url_param_split.next() {
                        Some(v) => v.to_string(),
                        None => break
                    };
                    let value = match url_param_split.next() {
                        Some(v) => v.to_string(),
                        None => break
                    };

                    request_header.url_parameters.insert(name, value);
                }
            }

            continue;
        }

        match *line_split.get(0).unwrap() {
            "Content-Type" => request_header.content_type = line_split.get(1).unwrap().to_string(),
            "Content-Length" => request_header.content_length = line_split.get(1).unwrap().parse().unwrap(),
            _ => (),
        }
    }

    request_header
}
