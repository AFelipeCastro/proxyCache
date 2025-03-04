use std::collections::HashMap;

// cache struct
#[derive(Debug)]
pub struct Cache {
    cache: HashMap<String, Vec<u8>>,
}

impl Cache {
    // create a new cache struct
    pub fn new() -> Self {
        Cache{ cache: HashMap::new()} 
    }
    
    // Gets the value assinged to the given key or None
    pub fn get(&self, k: &str) -> Option<&Vec<u8>> {
        self.cache.get(k) 
    }
    
    // Insert
    pub fn insert_pair(&mut self, k: String, v: Vec<u8>) {
        let _ = self.cache.insert(k, v);
    }

    // Get all keys
    pub fn get_keys(&self) -> Vec<&String>{
        self.cache.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_cache() {
        let _new_cache = Cache::new();
    }

    #[test]
    fn insert() {
        let mut new_cache = Cache::new();
        new_cache.insert_pair("HTTP/1.1 200 OK".to_string(), vec![b'h', b'o', b'l',b'a']);
        assert_eq!(new_cache.cache.get("HTTP/1.1 200 OK"), Some(&vec![b'h',b'o',b'l',b'a']));
    } 
}

