use brewer_core::{Brew, BrewBuilder, models::*};
use std::collections::HashSet;

#[test]
fn test_brew_construction() {
    let brew = Brew::default();
    assert!(!brew.path.as_os_str().is_empty());
    assert!(!brew.prefix.as_os_str().is_empty());
}

#[test]
fn test_brew_builder_construction() {
    let brew = BrewBuilder::default()
        .path("/custom/path/brew".into())
        .prefix("/custom/prefix".into())
        .build()
        .unwrap();
    
    assert_eq!(brew.path.to_str().unwrap(), "/custom/path/brew");
    assert_eq!(brew.prefix.to_str().unwrap(), "/custom/prefix");
}

#[test]
fn test_formula_creation() {
    let formula = formula::Formula {
        base: formula::base::Formula {
            name: "test-formula".to_string(),
            tap: "homebrew/core".to_string(),
            desc: Some("Test description".to_string()),
            homepage: Some("https://example.com".to_string()),
            caveats: None,
            build_dependencies: vec![],
            dependencies: vec!["dep1".to_string(), "dep2".to_string()],
            deprecated: false,
            deprecation_reason: None,
            disabled: false,
            disable_reason: None,
            aliases: {
                let mut set = HashSet::new();
                set.insert("alias1".to_string());
                set
            },
            versions: formula::base::Versions {
                stable: "1.2.3".to_string(),
                head: Some("HEAD".to_string()),
            },
        },
        executables: {
            let mut set = HashSet::new();
            set.insert("test-bin".to_string());
            set
        },
        analytics: None,
    };
    
    assert_eq!(formula.as_ref(), "test-formula");
    assert_eq!(formula.base.dependencies.len(), 2);
    assert_eq!(formula.executables.len(), 1);
}

#[test]
fn test_cask_creation() {
    let cask = cask::Cask {
        base: cask::base::Cask {
            token: "test-cask".to_string(),
            tap: "homebrew/cask".to_string(),
            names: { let mut set = HashSet::new(); set.insert("Test Cask".to_string()); set },
            desc: Some("Test cask description".to_string()),
            homepage: Some("https://example.com".to_string()),
            version: "2.0.0".to_string(),
            caveats: None,
            deprecated: false,
            deprecation_reason: None,
            disabled: false,
            disable_reason: None,
        },
    };
    
    assert_eq!(cask.base.token, "test-cask");
    assert_eq!(cask.base.version, "2.0.0");
    assert_eq!(cask.base.names.len(), 1);
}

#[test]
fn test_keg_enum_formula() {
    let formula = formula::Formula {
        base: formula::base::Formula {
            name: "test".to_string(),
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
                stable: "1.0.0".to_string(),
                head: None,
            },
        },
        executables: HashSet::new(),
        analytics: None,
    };
    
    let keg: Keg = formula.into();
    match keg {
        Keg::Formula(f) => assert_eq!(f.base.name, "test"),
        Keg::Cask(_) => panic!("Expected formula, got cask"),
    }
}

#[test]
fn test_keg_enum_cask() {
    let cask = cask::Cask {
        base: cask::base::Cask {
            token: "test".to_string(),
            tap: "homebrew/cask".to_string(),
            names: { let mut set = HashSet::new(); set.insert("Test".to_string()); set },
            desc: None,
            homepage: None,
            version: "1.0.0".to_string(),
            caveats: None,
            deprecated: false,
            deprecation_reason: None,
            disabled: false,
            disable_reason: None,
        },
    };
    
    let keg: Keg = cask.into();
    match keg {
        Keg::Cask(c) => assert_eq!(c.base.token, "test"),
        Keg::Formula(_) => panic!("Expected cask, got formula"),
    }
}

#[test]
fn test_state_structure() {
    use std::collections::HashMap;
    
    let state: State<HashMap<String, i32>, HashMap<String, String>> = State {
        formulae: {
            let mut map = HashMap::new();
            map.insert("key1".to_string(), 42);
            map
        },
        casks: {
            let mut map = HashMap::new();
            map.insert("key2".to_string(), "value".to_string());
            map
        },
    };
    
    assert_eq!(state.formulae.get("key1"), Some(&42));
    assert_eq!(state.casks.get("key2"), Some(&"value".to_string()));
}
