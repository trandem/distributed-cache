use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::time::Duration;
use log::warn;
use lru::LruCache;
use tokio::sync::RwLock;
use tokio::time::sleep;
use std::sync::Arc;


pub struct GlobalCache {
    num_shard: usize,
    shard_max_capacity: usize,
    datasource_center: Vec<RwLock<HashMap<Arc<i32>, Arc<Option<String>>>>>,
}

impl GlobalCache
{
    pub fn new(num_shard: usize, shard_max_capacity: usize) -> GlobalCache {
        if num_shard == 0 {
            panic!("num_shard is zero");
        }
        if shard_max_capacity == 0 {
            panic!("shard_max_capacity is zero");
        }
        let mut datasource_center = Vec::with_capacity(num_shard);
        for _ in 0..num_shard {
            datasource_center.push(RwLock::new(HashMap::new()));
        }

        GlobalCache {
            num_shard,
            shard_max_capacity,
            datasource_center,
        }
    }

    pub async fn get(&mut self, k: Arc<i32>) -> Option<Arc<Option<String>>> {
        let mut s = DefaultHasher::new();
        k.as_ref().hash(&mut s);
        let hash_key = s.finish();
        let shard = self.get_shard(hash_key);
        let datasource = self.datasource_center.get(shard).unwrap();
        let mut lru_cache = datasource.read().await;
        let value_option = lru_cache.get(k.as_ref());

        if value_option.is_some() {
            return Some(value_option.unwrap().clone());
        } else {
            drop(lru_cache);
            let mut lru_cache = datasource.write().await;
            // concurrent two or multithreading can reach here so we recheck key exited
            if lru_cache.contains_key(k.as_ref()) {
                let value_option = lru_cache.get_mut(k.as_ref());
                if value_option.is_none() {
                    return None;
                }
                return Some(value_option.unwrap().clone());
            }
            // one thread do loading data
            let value = self.find_value_internet(k.clone()).await;
            let value = Arc::new(value);
            lru_cache.insert(k.clone(), value);

            let value_option = lru_cache.get(k.as_ref());
            if value_option.is_none() {
                return None;
            }
            return Some(value_option.unwrap().clone());
        }
    }

    async fn find_value_internet(&self, k: Arc<i32>) -> Option<String> {
        sleep(Duration::from_millis(100)).await;
        Some("lol".to_string())
    }

    fn get_shard(&self, hash_key: u64) -> usize {
        let shard = hash_key % self.num_shard as u64;
        shard as usize
    }
}