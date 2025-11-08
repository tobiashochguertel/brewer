#[cfg(test)]
mod tests {
    use crate::store::{Store, State};
    use brewer_core::models::{formula, cask};
    use std::collections::{HashMap, HashSet};
    use tempfile::TempDir;

    fn create_test_store() -> (Store, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let store = Store::open(&db_path).unwrap();
        (store, temp_dir)
    }

    fn create_test_state() -> State {
        let mut formulae = HashMap::new();
        formulae.insert(
            "test-formula".to_string(),
            formula::Formula {
                base: formula::base::Formula {
                    name: "test-formula".to_string(),
                    tap: "homebrew/core".to_string(),
                    desc: Some("A test formula".to_string()),
                    homepage: Some("https://example.com".to_string()),
                    caveats: None,
                    build_dependencies: vec![],
                    dependencies: vec![],
                    deprecated: false,
                    deprecation_reason: None,
                    disabled: false,
                    disable_reason: None,
                    aliases: HashSet::new(),
                    versions: formula::base::Versions {
                        stable: "1.0.0".to_string(),
                        head: None,
                    },
                },
                executables: HashSet::new(),
                analytics: None,
            },
        );

        let mut casks = HashMap::new();
        casks.insert(
            "test-cask".to_string(),
            cask::Cask {
                base: cask::base::Cask {
                    token: "test-cask".to_string(),
                    tap: "homebrew/cask".to_string(),
                    names: { let mut set = HashSet::new(); set.insert("Test Cask".to_string()); set },
                    desc: Some("A test cask".to_string()),
                    homepage: Some("https://example.com".to_string()),
                    version: "1.0.0".to_string(),
                    caveats: None,
                    deprecated: false,
                    deprecation_reason: None,
                    disabled: false,
                    disable_reason: None,
                },
            },
        );

        State { formulae, casks }
    }

    #[test]
    fn test_store_open() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let result = Store::open(&db_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_store_last_update_empty() {
        let (store, _temp_dir) = create_test_store();
        
        let last_update = store.last_update().unwrap();
        assert!(last_update.is_none());
    }

    #[test]
    fn test_store_get_state_empty() {
        let (store, _temp_dir) = create_test_store();
        
        let state = store.get_state().unwrap();
        assert!(state.is_none());
    }

    #[test]
    fn test_store_set_and_get_state() {
        let (mut store, _temp_dir) = create_test_store();
        let test_state = create_test_state();
        
        // Set state
        let set_result = store.set_state(test_state.clone());
        assert!(set_result.is_ok());
        
        // Get state back
        let retrieved_state = store.get_state().unwrap();
        assert!(retrieved_state.is_some());
        
        let retrieved = retrieved_state.unwrap();
        assert_eq!(retrieved.formulae.len(), 1);
        assert_eq!(retrieved.casks.len(), 1);
        assert!(retrieved.formulae.contains_key("test-formula"));
        assert!(retrieved.casks.contains_key("test-cask"));
    }

    #[test]
    fn test_store_last_update_after_set() {
        let (mut store, _temp_dir) = create_test_store();
        let test_state = create_test_state();
        
        // Initially no update
        let before = store.last_update().unwrap();
        assert!(before.is_none());
        
        // Set state
        store.set_state(test_state).unwrap();
        
        // Now should have update time
        let after = store.last_update().unwrap();
        assert!(after.is_some());
    }

    #[test]
    fn test_store_update_state() {
        let (mut store, _temp_dir) = create_test_store();
        let mut test_state = create_test_state();
        
        // Set initial state
        store.set_state(test_state.clone()).unwrap();
        
        // Modify state
        test_state.formulae.insert(
            "another-formula".to_string(),
            formula::Formula {
                base: formula::base::Formula {
                    name: "another-formula".to_string(),
                    tap: "homebrew/core".to_string(),
                    desc: None,
                    homepage: None,
                    caveats: None,
                    build_dependencies: vec![],
                    dependencies: vec![],
                    deprecated: false,
                    deprecation_reason: None,
                    disabled: false,
                    disable_reason: None,
                    aliases: HashSet::new(),
                    versions: formula::base::Versions {
                        stable: "2.0.0".to_string(),
                        head: None,
                    },
                },
                executables: HashSet::new(),
                analytics: None,
            },
        );
        
        // Update state
        store.set_state(test_state).unwrap();
        
        // Verify update
        let retrieved = store.get_state().unwrap().unwrap();
        assert_eq!(retrieved.formulae.len(), 2);
    }

    #[test]
    fn test_store_clone() {
        let (store, _temp_dir) = create_test_store();
        let _cloned = store.clone();
        // Should compile and work
    }
}
