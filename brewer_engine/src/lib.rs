use std::time::Duration;

use chrono::Utc;
use derive_builder::Builder;

use brewer_core::{models, Brew};
use log::info;

use crate::store::Store;

pub mod store;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod lib_tests;

pub type State = models::State<models::formula::State, models::cask::State>;

#[derive(Builder)]
pub struct Engine {
    store: Store,

    #[builder(default)]
    brew: Brew,

    /// How often cache should expire. None means never
    cache_duration: Option<Duration>,
}

impl Engine {
    pub fn new(store: Store, brew: Brew) -> Engine {
        Engine {
            store,
            brew,
            cache_duration: Self::get_cache_ttl(),
        }
    }

    /// Get cache TTL from environment or default (24 hours)
    fn get_cache_ttl() -> Option<Duration> {
        // Check if cache should never expire
        if let Ok(val) = std::env::var("BREWER_CACHE_NEVER_EXPIRE") {
            if val == "1" || val.to_lowercase() == "true" {
                log::info!("Cache expiration disabled via BREWER_CACHE_NEVER_EXPIRE");
                return None;
            }
        }

        // Check for custom TTL in hours
        if let Ok(hours_str) = std::env::var("BREWER_CACHE_TTL") {
            if let Ok(hours) = hours_str.parse::<u64>() {
                log::info!("Using custom cache TTL: {} hours", hours);
                if hours == 0 {
                    log::info!("Cache TTL set to 0, will always fetch fresh data");
                }
                return Some(Duration::from_secs(hours * 3600));
            } else {
                log::warn!("Invalid BREWER_CACHE_TTL value: {}, using default", hours_str);
            }
        }

        // Default: 24 hours
        Some(Duration::from_secs(24 * 3600))
    }

    pub fn install(&self, kegs: Vec<models::Keg>) -> anyhow::Result<()> {
        self.brew.install(kegs)?;

        Ok(())
    }

    pub fn uninstall(&self, kegs: Vec<models::Keg>) -> anyhow::Result<()> {
        self.brew.uninstall(kegs)?;

        Ok(())
    }

    pub fn cache_or_latest(&mut self) -> anyhow::Result<State> {
        // Try to use cached brew JSON first
        if let Ok(Some(cached_json)) = self.store.get_cached_brew_json() {
            if !self.cache_expired()? {
                info!("using cached brew data");
                match self.brew.state_from_cache(&cached_json) {
                    Ok(state) => return Ok(state),
                    Err(e) => {
                        log::warn!("failed to use cached data: {}, fetching fresh", e);
                    }
                }
            } else {
                info!("cache expired, fetching fresh data");
            }
        } else {
            info!("no cache available, fetching fresh data (this will take 2-3 minutes)");
        }

        // Fetch fresh data and cache the raw JSON
        let (state, json_data) = self.brew.fetch_and_get_json()?;
        
        // Cache the raw JSON for fast future loads
        self.store.cache_brew_json(&json_data)?;
        
        // Also update the traditional state cache
        self.update_cache(&state)?;

        Ok(state)
    }

    pub fn cache(&self) -> anyhow::Result<Option<State>> {
        let Some(all) = self.store.get_state()? else {
            return Ok(None);
        };

        let installed = self.brew.installed(&all)?;

        let state = State {
            formulae: models::formula::State {
                all: all.formulae,
                installed: installed.formulae,
            },
            casks: models::cask::State {
                all: all.casks,
                installed: installed.casks,
            },
        };

        Ok(Some(state))
    }

    pub fn cache_expired(&self) -> anyhow::Result<bool> {
        let Some(cache_duration) = self.cache_duration else {
            // No expiration set (BREWER_CACHE_NEVER_EXPIRE=1)
            log::debug!("Cache never expires");
            return Ok(false);
        };

        // If TTL is 0, always consider expired (force fresh)
        if cache_duration.as_secs() == 0 {
            log::info!("Cache TTL is 0, forcing fresh fetch");
            return Ok(true);
        }

        let last_update = self.store.last_update()?;

        match last_update {
            Some(last_update) => {
                let now = Utc::now().naive_utc();
                let expired = last_update + cache_duration <= now;

                if expired {
                    let age_secs = (now - last_update).num_seconds();
                    log::info!("Cache expired: age {}s > TTL {}s", 
                        age_secs, cache_duration.as_secs());
                }

                Ok(expired)
            }
            None => Ok(true),
        }
    }

    pub fn update_cache(&mut self, state: &State) -> anyhow::Result<()> {
        self.store.set_state(store::State {
            formulae: state.formulae.all.clone(),
            casks: state.casks.all.clone(),
        })?;

        Ok(())
    }

    pub fn fetch_latest(&self) -> anyhow::Result<State> {
        let state = self.brew.state()?;

        Ok(state)
    }
}

