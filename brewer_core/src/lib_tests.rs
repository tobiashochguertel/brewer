#[cfg(test)]
mod tests {
    use crate::{Brew, BrewBuilder, split_kegs, models::*};
    use std::collections::HashSet;

    #[test]
    fn test_brew_default() {
        let brew = Brew::default();
        
        assert!(brew.path.to_str().is_some());
        assert!(brew.prefix.to_str().is_some());
    }

    #[test]
    fn test_brew_builder() {
        let brew = BrewBuilder::default()
            .path("/usr/local/bin/brew".into())
            .prefix("/usr/local".into())
            .build()
            .unwrap();
        
        assert_eq!(brew.path.to_str().unwrap(), "/usr/local/bin/brew");
        assert_eq!(brew.prefix.to_str().unwrap(), "/usr/local");
    }

    #[test]
    fn test_split_kegs_empty() {
        let kegs = Vec::new();
        let (formulae, casks) = split_kegs(kegs);
        
        assert!(formulae.is_empty());
        assert!(casks.is_empty());
    }

    #[test]
    fn test_split_kegs_with_formula() {
        let formula = formula::Formula {
            base: formula::base::Formula {
                name: "test-formula".to_string(),
                tap: "homebrew/core".to_string(),
                desc: Some("Test formula".to_string()),
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
        
        let kegs = vec![Keg::Formula(formula.clone())];
        let (formulae, casks) = split_kegs(kegs);
        
        assert_eq!(formulae.len(), 1);
        assert!(casks.is_empty());
        assert_eq!(formulae[0].base.name, "test-formula");
    }

    #[test]
    fn test_split_kegs_with_cask() {
        let cask = cask::Cask {
            base: cask::base::Cask {
                token: "test-cask".to_string(),
                tap: "homebrew/cask".to_string(),
                names: { let mut set = HashSet::new(); set.insert("Test Cask".to_string()); set },
                desc: Some("Test cask".to_string()),
                homepage: Some("https://example.com".to_string()),
                version: "1.0.0".to_string(),
                caveats: None,
                deprecated: false,
                deprecation_reason: None,
                disabled: false,
                disable_reason: None,
            },
        };
        
        let kegs = vec![Keg::Cask(cask.clone())];
        let (formulae, casks) = split_kegs(kegs);
        
        assert!(formulae.is_empty());
        assert_eq!(casks.len(), 1);
        assert_eq!(casks[0].base.token, "test-cask");
    }

    #[test]
    fn test_split_kegs_mixed() {
        let formula = formula::Formula {
            base: formula::base::Formula {
                name: "test-formula".to_string(),
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
        
        let cask = cask::Cask {
            base: cask::base::Cask {
                token: "test-cask".to_string(),
                tap: "homebrew/cask".to_string(),
                names: { let mut set = HashSet::new(); set.insert("Test Cask".to_string()); set },
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
        
        let kegs = vec![Keg::Formula(formula), Keg::Cask(cask)];
        let (formulae, casks) = split_kegs(kegs);
        
        assert_eq!(formulae.len(), 1);
        assert_eq!(casks.len(), 1);
    }

    #[test]
    fn test_keg_from_formula() {
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
        assert!(matches!(keg, Keg::Formula(_)));
    }

    #[test]
    fn test_keg_from_cask() {
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
        assert!(matches!(keg, Keg::Cask(_)));
    }

    #[test]
    fn test_formula_as_ref() {
        let formula = formula::Formula {
            base: formula::base::Formula {
                name: "test-name".to_string(),
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
        
        assert_eq!(formula.as_ref(), "test-name");
    }
}
