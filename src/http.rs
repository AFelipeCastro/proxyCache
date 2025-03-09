use async_std::{net::{self, TcpStream, ToSocketAddrs}, io::WriteExt};

// This sends an http request to origin
// Takes:
//  - Buffer (try to pass an entire slice and a slice where it "ends")
//  - Addr
// Returns:
//  - Stream 
pub async fn forward_to(buf: &[u8], origin: &str) -> async_native_tls::TlsStream<net::TcpStream> {
        
    let addr = (origin, 443)
        .to_socket_addrs()
        .await
        .unwrap()
        .next()
        .unwrap();

    let stream = TcpStream::connect(addr).await.unwrap();
    let mut stream = async_native_tls::connect(origin, stream).await.unwrap();
    let _ = stream.write(buf).await.unwrap();
    
    stream
}

// Takes:
// - request buffer
// Returns:
// - Vector containing request/response headers
pub fn get_headers(buffer: &[u8]) -> Vec<u8> {
    let mut count: u8 = 0;

    buffer
        .iter()
        .map_while(|&char| {
            match char.is_ascii_control() {
                true => {
                        if count == 2 {
                            None
                        } else {
                            count += 1;
                            Some(char)
                        }
                },
                false => { 
                    count = 0;
                    Some(char)
                },
            }
        })
        .collect()

}

//
//
pub fn get_content_len(buffer: &[u8]) -> Option<usize> {
    let headers = get_headers(buffer);
    if let Some(len) = find_header_val(&headers, "Content-length") {
        println!("debug get_content_len: {len}");
        len.parse().ok() 
    } else {
        None
    }
}

// Extract:
//  - Request line
//  - Host
// Takes: 
//  - String with request line and headers (request headers)
pub fn get_req_data(req: &[u8]) -> (String, String) {

    let headers = get_headers(req);
    
    let req_line = std::str::from_utf8(&headers)
        .unwrap()
        .split_terminator("\r\n")
        .nth(0)
        .map(|line| line.to_string())
        .unwrap();

    let host = find_header_val(&headers[..], "Host").unwrap();
    
    (req_line, host)
}

pub fn find_header_val(headers: &[u8], key: &str) -> Option<String> {
    std::str::from_utf8(headers)
        .unwrap()
        .split_terminator("\r\n")
        .find(|header| header.starts_with(key))
        .and_then(|header| {
            header
                .split_terminator(": ")
                .nth(1)
                .map(|s| s.to_string())
        })
}

// Returns a slice of the body
// Takes:
//  - &buffer[..bytes_read]
// Returns:
//  - &[u8]
pub fn get_body(buffer: &[u8]) -> &[u8] {
    let mut counter: u8 = 0;

    let start = buffer
        .iter()
        .position(|byte| {
            match byte.is_ascii_control() {
                true => {
                    if counter == 2 {
                        true
                    } else {
                        counter += 1;
                        false
                    }
                },
                false => {
                    counter = 0;
                    false
                },
            }
        })
        .unwrap();
    &buffer[start..]
}


// Adds a header to the response
// Takes: 
//  - Buffer
//  - new header as str
// Returns:
//  - New buffer containig new headers
// It will separate the headers and the body
//  will add the new_header and join
//  the headers with the body
pub fn add_header(buffer: Vec<u8>, header: &str) -> Vec<u8>{
    let mut headers = get_headers(&buffer[..]);
    let body = get_body(&buffer[..]);
    let add = header.to_string() + "\r\n";
    headers.extend_from_slice(add.as_bytes());
    headers.extend_from_slice(body);
    headers
}
