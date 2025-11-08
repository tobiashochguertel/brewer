#[cfg(test)]
mod tests {
    use crate::{Engine, EngineBuilder, store::Store};
    use brewer_core::Brew;
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_test_store() -> (Store, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Store::open(&db_path).unwrap();
        (store, temp_dir)
    }

    #[test]
    fn test_engine_new() {
        let (store, _temp_dir) = create_test_store();
        let brew = Brew::default();
        let engine = Engine::new(store, brew);
        
        // Engine should be created successfully
        assert!(engine.cache().is_ok());
    }

    #[test]
    fn test_engine_builder() {
        let (store, _temp_dir) = create_test_store();
        let brew = Brew::default();
        
        let engine = EngineBuilder::default()
            .store(store)
            .brew(brew)
            .cache_duration(Some(Duration::from_secs(3600)))
            .build()
            .unwrap();
        
        assert!(engine.cache().is_ok());
    }

    #[test]
    fn test_engine_cache_empty() {
        let (store, _temp_dir) = create_test_store();
        let brew = Brew::default();
        let engine = Engine::new(store, brew);
        
        let result = engine.cache().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_expired_no_duration() {
        let (store, _temp_dir) = create_test_store();
        let brew = Brew::default();
        let engine = Engine::new(store, brew);
        
        let expired = engine.cache_expired().unwrap();
        assert!(!expired); // Should never expire if no duration set
    }

    #[test]
    fn test_cache_expired_with_duration_no_cache() {
        let (store, _temp_dir) = create_test_store();
        let brew = Brew::default();
        
        let engine = EngineBuilder::default()
            .store(store)
            .brew(brew)
            .cache_duration(Some(Duration::from_secs(3600)))
            .build()
            .unwrap();
        
        let expired = engine.cache_expired().unwrap();
        assert!(expired); // Should be expired if no cache exists
    }
}
