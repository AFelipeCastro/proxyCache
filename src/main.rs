use async_std::{io::{ReadExt, WriteExt}, net, prelude::*, sync::{Arc, RwLock}, task};
use std::str;

mod cache;
mod http;


// this should include only the main logic of: checking cache
// forward to server or client using http mod
// clearing cache using TTL Takes: - TTL This is a task that will run in the background and in intervals given in seconds by TTL.
//async fn clear_cache(ttl: time::Duration) {
 //   todo!()
//}

// Takes:
//  - The stream of the connection
//  - The lock on the cache
// Extract the request line (using http)
// Check if Cache struct contains any key that corresponds to this request line.
// If so send the request in cache witht the header Cache: hit
// Else forward the request (using http). receive the response, 
//  add the header Cache: miss and forward it to the client.
async fn handle_conn(mut stream: async_std::net::TcpStream, cache: Arc<RwLock<cache::Cache>>, ttl: u64) {
    let mut buf_req: Vec<u8> = vec![0;2024];
    let bytes_read = stream.read(&mut buf_req).await.unwrap();
    
    let (req_line, host) = http::get_req_data(&buf_req);
    
    let  cache_read = cache.read_arc().await; 

    if let Some(response) = cache_read.get(&req_line)
    {
        if response.is_fresh() {
            let fin_buf = http::add_header(response.get_buffer().to_owned(), "Cache: hit");
            let _ = stream.write(&fin_buf[..]).await.unwrap();
        } else {
            println!("isn't fresh or in cache");
        }
    } else {
        let mut stream_host = http::forward_to(&buf_req[..bytes_read], &host).await;
        
        let mut read_buff: Vec<u8> = Vec::new();
        loop {
            let origin_read = match stream_host.read(&mut buf_req).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    println!("Error reading from stream: {e}");
                    break;
                },
            };

            println!("read: {origin_read}");

            read_buff.extend_from_slice(&buf_req[..origin_read]);

            if let Some(content_length) = http::get_content_len(&read_buff[..]) {
                println!("content_length: {content_length}");
                if read_buff.len() >= content_length {
                    break;
                }
            } else {
                println!("Hola");
            }
        }

        drop(cache_read); // drop read lock

        if cache::is_cachable(&read_buff[..]) {
            let new_cached = cache::CachedResponse::new(&read_buff[..], ttl);
            let mut cache_write = cache.write_arc().await;
            cache_write.insert_pair(req_line, new_cached);

            let fin_buff = http::add_header(read_buff, "Cache: miss");
            let _ = stream.write(&fin_buff[..]).await.unwrap();
        } else {
            let fin_buff = http::add_header(read_buff, "Cache: miss");
            let _ = stream.write(&fin_buff[..]).await.unwrap();
        }
    }
}
// Main loop that will run until sig INT Takes:
//  - Port
//  - Lock on cache
// Will spawn a task that will manage a single connection
async fn accept_loop(cache: Arc<RwLock<cache::Cache>>, port: &str, ttl: u64) {
    let address = format!("127.0.0.1:{port}");
    let server = net::TcpListener::bind(address).await.expect("Couldn't bind");
    let mut incoming = server.incoming();

    while let Some(stream) = incoming.next().await {
        let stream = stream.unwrap(); // TODO use or_else with a custom function to log errors to file
        let _ = task::spawn(handle_conn(stream, Arc::clone(&cache), ttl));
    }
}

// Starting point
// 
// Takes command line arguments:
//  - Port
//  - TTL
//
// Initiates a main loop which will be a blocking task
//
// Will be in charge of creating the cache struct.
// This struct must be shared by multiple owners, be thread
// safe and avoid data races.
fn main() {
    let args: Vec<String> = std::env::args().collect();

    let cache_lock = Arc::new(RwLock::new(cache::Cache::new()));
    
    let _ = task::block_on(accept_loop(cache_lock,&args[1],args[2].parse().unwrap()));
}

// proxy cache
//
// in this server the client will send requests
// the proxy will forward those requests and receive the response
// it will save the response in cache and forward the response 
// (adding a Cache: miss header)
// the next time the client makes the same request the proxy
// will anwer with the response it has in cache and add a Cache: hit 
