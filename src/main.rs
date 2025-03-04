use async_std::{io::{ReadExt, WriteExt}, net, prelude::*, task, sync::{RwLock, Arc}};
use std::str;

mod cache;
mod http;

// this should include only the main logic of: checking cache
// forward to server or client using http mod
// clearing cache using TTL 

// Takes:
//  - TTL
// This is a task that will run in the background and in intervals given in
//  seconds by TTL.
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
async fn handle_conn(mut stream: async_std::net::TcpStream, cache: Arc<RwLock<cache::Cache>>) {
    let mut buf_req: Vec<u8> = vec![0;1024];
    let bytes_read = stream.read(&mut buf_req).await.unwrap();
    
    let (req_line, host) = http::get_req_data(&buf_req);
    
    let  cache_read = cache.read_arc().await; 

    if let Some(response) = cache_read.get(&req_line)
    {
        println!("Cache: hit");
        // add heaader Cache: hit
        let _ = stream.write(response).await.unwrap();
    } else {
        // add header Cache: miss
        let mut stream_host = http::forward_to(&buf_req[..bytes_read], &host).await;

        let host_read = stream_host.read(&mut buf_req).await.unwrap();

        drop(cache_read); // drop read lock

        let mut cache_write = cache.write_arc().await;
        cache_write.insert_pair(req_line, buf_req[..host_read].to_owned());
        
        //debug
        println!("hash map: {:?}", cache_write.get_keys());
        // Write to the origin
        let _ = stream.write(&buf_req[..host_read]).await.unwrap();
        println!("Cache: miss");
    }
}
// Main loop that will run until sig INT Takes:
//  - Port
//  - Lock on cache
// Will spawn a task that will manage a single connection
async fn accept_loop(cache: Arc<RwLock<cache::Cache>>, port: &str) {
    let address = format!("127.0.0.1:{port}");
    let server = net::TcpListener::bind(address).await.expect("Couldn't bind");
    let mut incoming = server.incoming();

    while let Some(stream) = incoming.next().await {
        let stream = stream.unwrap(); // TODO use or_else with a custom function to log errors to file
        let _ = task::spawn(handle_conn(stream, Arc::clone(&cache)));
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
    
    let _ = task::block_on(accept_loop(cache_lock,&args[1]));
}

// proxy cache
//
// in this server the client will send requests
// the proxy will forward those requests and receive the response
// it will save the response in cache and forward the response 
// (adding a Cache: miss header)
// the next time the client makes the same request the proxy
// will anwer with the response it has in cache and add a Cache: hit 
