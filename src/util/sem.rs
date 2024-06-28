use std::collections::HashMap;

use tokio::sync::{Semaphore, SemaphorePermit};

#[derive(Debug)]
pub struct Lock {
    lk: HashMap<String, Semaphore>,
}

impl Lock {
    pub fn new(keys: &[&str]) -> Self {
        let mut lk = HashMap::new();
        for key in keys {
            lk.insert(key.to_string(), Semaphore::new(0));
        }
        Self {
            lk
        }
    }
    pub async fn ready(&self, key: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.lk.get(&key.to_string())
            .ok_or("lock key does not exist")?
            .add_permits(Semaphore::MAX_PERMITS);
        Ok(())
    }
    pub async fn ready_size(&self, key: &str, limit: usize) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.lk.get(&key.to_string())
            .ok_or("lock key does not exist")?
            .add_permits(limit);
        Ok(())
    }
    pub async fn access(&self, key: &str) -> Result<SemaphorePermit, Box<dyn std::error::Error + Send + Sync>> {
        let sem = self.lk.get(&key.to_string())
            .ok_or("lock key does not exist")?
            .acquire()
            .await?; 
        Ok(sem)
    }
}

