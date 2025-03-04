use async_std::net::TcpStream;
// here will have code related to http
use async_std::{io::WriteExt, net, net::ToSocketAddrs};


// This sends an http request to a host
// Takes:
//  - Buffer (try to pass an entire slice and a slice where it "ends")
//  - Addr
// Returns:
//  - Stream 
pub async fn forward_to(buf: &[u8], addr: &str) -> net::TcpStream {
    let addr = (addr, 80)
        .to_socket_addrs()
        .await
        .unwrap()
        .next()
        .unwrap();

    let mut stream = TcpStream::connect(addr).await.unwrap();
    let _ = stream.write(buf).await.unwrap();
    stream
}

// Extract:
//  - Request line
pub fn get_req_data(req: &Vec<u8>) -> (String, String) {
    // perform checks:
    //  - Starts with Method
    //  - Contains URI
    //  - Ends with protocol version HTTP/1.1
    // If so then 
    //  Extract the line with terminator "\r\n"
    //  Convert it to String using from-utf8 // I don't know the consequesnces of &str
    // else
    //  Just panicked. In the future implement send HTTP error

    let mut count: u8 = 0;

    let headers = req
        .iter()
        .map_while(|&char| {
            match count {
                3.. => None,
                _ => {
                    if char.is_ascii_control() {
                        count += 1;
                    } else {
                        count = 0;
                    }
                    Some(char as char)
                },
            }
        })
        .collect::<String>();

    let mut headers_iter = headers.split_terminator("\r\n");

    let req_line = headers_iter
        .next()
        .unwrap()
        .to_string();

    let host: String = headers_iter
        .find(|header| header.starts_with("Host"))
        .and_then(|header| {
            header
                .split_terminator(": ")
                .nth(1) // Get the second part after ": "
                .map(|s| s.to_string())
        })
        .unwrap();

    (req_line, host)

}

// Adds the header cache to response
// Takes: 
//  - Response buff
//  - Boolean saying if hit or miss
//pub fn add_cache_header() {
 //   todo!()
//}
