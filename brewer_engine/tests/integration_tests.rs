use brewer_engine::{Engine, EngineBuilder, store::Store};
use brewer_core::Brew;
use tempfile::TempDir;
use std::time::Duration;

fn setup_test_engine() -> (Engine, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let store = Store::open(&db_path).unwrap();
    let brew = Brew::default();
    
    let engine = EngineBuilder::default()
        .store(store)
        .brew(brew)
        .cache_duration(Some(Duration::from_secs(3600)))
        .build()
        .unwrap();
    
    (engine, temp_dir)
}

#[test]
fn test_engine_creation_and_empty_cache() {
    let (engine, _temp_dir) = setup_test_engine();
    
    let cache = engine.cache().unwrap();
    assert!(cache.is_none(), "Cache should be empty initially");
}

#[test]
fn test_engine_cache_expiration() {
    let (engine, _temp_dir) = setup_test_engine();
    
    let is_expired = engine.cache_expired().unwrap();
    assert!(is_expired, "Cache should be expired when empty");
}

#[test]
fn test_engine_with_no_cache_duration() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let store = Store::open(&db_path).unwrap();
    let brew = Brew::default();
    
    let engine = Engine::new(store, brew);
    
    let is_expired = engine.cache_expired().unwrap();
    assert!(!is_expired, "Cache should never expire without duration");
}
