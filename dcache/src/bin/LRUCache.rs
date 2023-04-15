use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;


pub struct DemoCache {
    cache: LruCache<Arc<String>, String>,
}

impl DemoCache {
    fn new() -> DemoCache {
        DemoCache {
            cache: LruCache::new(NonZeroUsize::new(100).unwrap())
        }
    }
    fn get_or_load(&mut self, key: Arc<String>) {
        self.cache.put(key,"lol".to_string());
    }
}

fn main() {
    let mut demo_cache = DemoCache::new();
    let key = Arc::new("ss".to_string());
    demo_cache.get_or_load(key.clone());

    let x = demo_cache.cache.get(key.as_ref());
    println!("{:?}",x);
    let x = demo_cache.cache.get(key.as_ref());

}