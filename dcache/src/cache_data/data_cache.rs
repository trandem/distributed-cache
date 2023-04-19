use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use log::info;
use tokio::sync::RwLock;
use tokio::time::sleep;
use futures::channel::oneshot;
use futures::channel::oneshot::Receiver;
use futures::future::Shared;
use futures::FutureExt;

use crate::cache_data::dto::KeyValue;
use tokio::sync::mpsc::channel;


pub struct GlobalCache {
    num_shard: usize,
    shard_max_capacity: usize,
    datasource_center: Vec<RwLock<HashMap<i32, Shared<Receiver<String>>>>>,
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

    pub async fn get(&self, k: i32) -> Option<Shared<Receiver<String>>> {
        let shard = self.get_shard(k);
        let datasource = self.datasource_center.get(shard).unwrap();
        let mut lru_cache = datasource.read().await;
        let value_option = lru_cache.get(&k);

        if value_option.is_some() {
            return Some(value_option.unwrap().clone());
        } else {
            drop(lru_cache);
            let mut lru_cache = datasource.write().await;
            // concurrent two or multithreading can reach here so we recheck key exited
            if lru_cache.contains_key(&k) {
                let value_option = lru_cache.get_mut(&k);
                if value_option.is_none() {
                    return None;
                }
                return Some(value_option.unwrap().clone());
            }
            // one thread do loading data
            let value = self.find_value_internet(k).await;
            lru_cache.insert(k, value.shared());

            let value_option = lru_cache.get(&k);
            if value_option.is_none() {
                return None;
            }
            return Some(value_option.unwrap().clone());
        }
    }

    pub async fn invalid(&self, k: i32) -> Option<Shared<Receiver<String>>> {
        let shard = self.get_shard(k);
        let datasource = self.datasource_center.get(shard).unwrap();
        let mut lru_cache = datasource.write().await;
        lru_cache.remove(&k)
    }


    async fn find_value_internet(&self, k: i32) -> Receiver<String> {
        let (sender, receiver) = oneshot::channel::<String>();
        tokio::spawn(async {
            sleep(Duration::from_millis(100)).await;
            info!("get from internet");
            let mut value = String::new();
            value.push_str("lol_");
            let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
            info!("{}", time);
            value.push_str(time.to_string().as_str());
            sender.send(value)
        });

        receiver
    }

    pub async fn find_values_on_internet(&self, keys: Vec<i32>) -> tokio::sync::mpsc::Receiver<KeyValue> {
        let (sender, receiver) = channel(keys.len());

        for key in keys {
            let sender_clone = sender.clone();
            tokio::spawn(async move {
                sleep(Duration::from_millis(100)).await;
                info!("get from internet in thread for key {}", key);

                let mut value = String::new();
                value.push_str("lol_");
                let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                info!("{}", time);
                value.push_str(time.to_string().as_str());

                sender_clone.send(KeyValue {
                    key,
                    value,
                }).await;
            });
        }

        receiver
    }

    fn get_shard(&self, k: i32) -> usize {
        let mut s = DefaultHasher::new();
        k.hash(&mut s);
        let hash_key = s.finish();
        let shard = hash_key % self.num_shard as u64;
        shard as usize
    }

    pub async fn is_key_exist(&self, k: i32) -> bool {
        let shard = self.get_shard(k);
        let datasource = self.datasource_center.get(shard).unwrap();
        let lru_cache = datasource.read().await;
        lru_cache.contains_key(&k)
    }
}