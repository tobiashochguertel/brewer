use std::path::Path;

use chrono::{NaiveDateTime, Utc};
use jammdb::Tx;

use brewer_core::models;

#[derive(Clone)]
pub struct Store {
    db: jammdb::DB,
}

pub type State = models::State<models::formula::Store, models::cask::Store>;

#[cfg(test)]
#[path = "store_tests.rs"]
mod store_tests;

impl Store {
    const UPDATE_BUCKET: &'static str = "update";
    const STATE_BUCKET: &'static str = "state";
    const BREW_CACHE_BUCKET: &'static str = "brew_cache";

    const STATE_KEY: &'static str = "state";
    const BREW_JSON_KEY: &'static str = "brew_json_v1";
    const EXECUTABLES_KEY: &'static str = "executables_v1";
    const EXECUTABLES_UPDATE_KEY: &'static str = "executables_update";

    pub fn open(path: &Path) -> anyhow::Result<Store> {
        Ok(Store {
            db: jammdb::DB::open(path)?
        })
    }

    pub fn last_update(&self) -> anyhow::Result<Option<NaiveDateTime>> {
        let tx = self.db.tx(false)?;

        match tx.get_bucket(Self::UPDATE_BUCKET) {
            Ok(bucket) => {
                let Some(data) = bucket.get(Self::STATE_KEY) else {
                    return Ok(None);
                };

                let datetime: NaiveDateTime = rmp_serde::from_slice(data.kv().value())?;


                Ok(Some(datetime))
            }
            Err(jammdb::Error::BucketMissing) => Ok(None),
            Err(e) => Err(anyhow::anyhow!(e))
        }
    }

    fn commit_update(tx: Tx) -> anyhow::Result<()> {
        let bucket = tx.get_or_create_bucket(Self::UPDATE_BUCKET)?;

        let now = Utc::now().naive_utc();
        let now_bytes = rmp_serde::to_vec(&now)?;

        bucket.put(Self::STATE_KEY, now_bytes)?;

        tx.commit()?;

        Ok(())
    }

    pub fn get_state(&self) -> anyhow::Result<Option<State>> {
        let tx = self.db.tx(false)?;

        match tx.get_bucket(Self::STATE_BUCKET) {
            Ok(bucket) => {
                let Some(data) = bucket.get(Self::STATE_KEY) else {
                    return Ok(None);
                };

                let state: State = rmp_serde::from_slice(data.kv().value())?;

                Ok(Some(state))
            }
            Err(jammdb::Error::BucketMissing) => Ok(None),
            Err(e) => Err(anyhow::anyhow!(e))
        }
    }

    pub fn set_state(&mut self, state: State) -> anyhow::Result<()> {
        let tx = self.db.tx(true)?;

        let bucket = tx.get_or_create_bucket(Self::STATE_BUCKET)?;

        let state_bytes = rmp_serde::to_vec(&state)?;

        bucket.put(Self::STATE_KEY, state_bytes)?;

        Self::commit_update(tx)?;

        Ok(())
    }

    /// Cache raw brew JSON output for faster subsequent loads
    pub fn cache_brew_json(&mut self, json_data: &[u8]) -> anyhow::Result<()> {
        let tx = self.db.tx(true)?;
        let bucket = tx.get_or_create_bucket(Self::BREW_CACHE_BUCKET)?;
        
        bucket.put(Self::BREW_JSON_KEY, json_data)?;
        
        Self::commit_update(tx)?;
        Ok(())
    }

    /// Get cached brew JSON output
    pub fn get_cached_brew_json(&self) -> anyhow::Result<Option<Vec<u8>>> {
        let tx = self.db.tx(false)?;

        match tx.get_bucket(Self::BREW_CACHE_BUCKET) {
            Ok(bucket) => {
                let Some(data) = bucket.get(Self::BREW_JSON_KEY) else {
                    return Ok(None);
                };

                Ok(Some(data.kv().value().to_vec()))
            }
            Err(jammdb::Error::BucketMissing) => Ok(None),
            Err(e) => Err(anyhow::anyhow!(e))
        }
    }

    /// Get size of cached data in bytes
    pub fn cache_size(&self) -> anyhow::Result<usize> {
        match self.get_cached_brew_json()? {
            Some(data) => Ok(data.len()),
            None => Ok(0),
        }
    }

    /// Clear the brew JSON cache
    pub fn clear_brew_cache(&mut self) -> anyhow::Result<()> {
        let tx = self.db.tx(true)?;
        
        // Delete and recreate the bucket to clear it
        if tx.get_bucket(Self::BREW_CACHE_BUCKET).is_ok() {
            tx.delete_bucket(Self::BREW_CACHE_BUCKET)?;
        }
        
        tx.commit()?;
        Ok(())
    }

    /// Cache executables data
    pub fn cache_executables(&mut self, data: &str) -> anyhow::Result<()> {
        let tx = self.db.tx(true)?;
        let bucket = tx.get_or_create_bucket(Self::BREW_CACHE_BUCKET)?;
        
        bucket.put(Self::EXECUTABLES_KEY, data.as_bytes())?;
        
        // Update executables timestamp
        let now = Utc::now().naive_utc();
        let now_bytes = rmp_serde::to_vec(&now)?;
        bucket.put(Self::EXECUTABLES_UPDATE_KEY, now_bytes)?;
        
        tx.commit()?;
        Ok(())
    }

    /// Get cached executables data
    pub fn get_cached_executables(&self) -> anyhow::Result<Option<String>> {
        let tx = self.db.tx(false)?;

        match tx.get_bucket(Self::BREW_CACHE_BUCKET) {
            Ok(bucket) => {
                let Some(data) = bucket.get(Self::EXECUTABLES_KEY) else {
                    return Ok(None);
                };

                let text = String::from_utf8(data.kv().value().to_vec())
                    .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in cached executables: {}", e))?;
                
                Ok(Some(text))
            }
            Err(jammdb::Error::BucketMissing) => Ok(None),
            Err(e) => Err(anyhow::anyhow!(e))
        }
    }

    /// Get last update time for executables
    pub fn executables_last_update(&self) -> anyhow::Result<Option<NaiveDateTime>> {
        let tx = self.db.tx(false)?;

        match tx.get_bucket(Self::BREW_CACHE_BUCKET) {
            Ok(bucket) => {
                let Some(data) = bucket.get(Self::EXECUTABLES_UPDATE_KEY) else {
                    return Ok(None);
                };

                let datetime: NaiveDateTime = rmp_serde::from_slice(data.kv().value())?;
                Ok(Some(datetime))
            }
            Err(jammdb::Error::BucketMissing) => Ok(None),
            Err(e) => Err(anyhow::anyhow!(e))
        }
    }

    /// Clear executables cache
    pub fn clear_executables_cache(&mut self) -> anyhow::Result<()> {
        let tx = self.db.tx(true)?;
        
        if let Ok(bucket) = tx.get_bucket(Self::BREW_CACHE_BUCKET) {
            // Remove executables keys
            let _ = bucket.delete(Self::EXECUTABLES_KEY);
            let _ = bucket.delete(Self::EXECUTABLES_UPDATE_KEY);
        }
        
        tx.commit()?;
        Ok(())
    }
}