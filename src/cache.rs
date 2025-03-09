use std::{collections::HashMap, time::{UNIX_EPOCH, SystemTime, Duration}};
use chrono::prelude::*;

use crate::http;

// cache struct
pub struct Cache {
    cache: HashMap<String, CachedResponse>,
}

pub struct CachedResponse{
    pub response: Vec<u8>,
        pub date: SystemTime, 
      pub expire: SystemTime,
}

impl CachedResponse {
    pub fn new(buff: &[u8], ttl: u64) -> Self {
        let headers = http::get_headers(&buff.to_vec());
        let date = SystemTime::now();
        
        let mut expire = SystemTime::now();

        if let Some(value) = http::find_header_val(&headers, "Cache-control") {
            let max_age = value
                .split_terminator(", ") 
                .find(|dir| {
                    dir.starts_with("max-age") 
                })
                .map(|val| {
                    val
                        .split_terminator("=")
                        .nth(1)
                        .map(|dur| {
                            dur
                                .parse::<u64>()
                                .unwrap()
                        })
                        .unwrap()
                })
                .unwrap();
            expire += Duration::new(max_age, 0);

        } else if let Some(value) = http::find_header_val(&headers, "Expires") {
            let expire_timestamp: u64 = DateTime::parse_from_rfc2822(&value)
                .unwrap()
                .timestamp()
                .try_into()
                .unwrap();
            expire = UNIX_EPOCH + Duration::new(expire_timestamp, 0);
        } else {
            expire += Duration::new(ttl, 0);
        }

        CachedResponse {
            response: buff.to_vec(),
            date,  
            expire,
        }
    }

    pub fn is_fresh(&self) -> bool {
        self.date <= self.expire 
    }

    pub fn get_buffer(&self) -> &Vec<u8> {
        &self.response
    }
}
// New struct (will be value in HashMap)
//  - Response Vec<u8>
//  - Date
//  - Max-age
//  - Expire date
//  - Implement method is_fresh
//      this should check if the cached response
//      is fresh based on the fields Date,
//      Max-age/Expire
//  - ?Implement method new?
//      this should set the fields based on the
//      body of the response

impl Cache {
    // create a new cache struct
    pub fn new() -> Self {
        Cache{ cache: HashMap::new()} 
    }
    
    // Gets the value assinged to the given key or None
    pub fn get(&self, k: &str) -> Option<&CachedResponse> {
        self.cache.get(k) 
    }
    
    // Insert
    pub fn insert_pair(&mut self, k: String, v: CachedResponse) {
        let _ = self.cache.insert(k, v);
    }

}

pub fn is_cachable(response: &[u8]) -> bool {
    let headers = http::get_headers(response);
    match http::find_header_val(&headers, "Cache-Control") {
        Some(value) => {
            if let Some(_) = value
                .split_terminator(", ")
                .find_map(|val| {
                    if val == "no-store" {
                        return Some(true)
                    }
                    None
                }) {
                return false
            }
            else {
                true
            }
        }
        None => true, 
    }
}
