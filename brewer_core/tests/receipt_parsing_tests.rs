use brewer_core::{Brew, BrewBuilder};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_brew_with_prefix(prefix: PathBuf) -> Brew {
    BrewBuilder::default()
        .path("/usr/bin/true".into()) // Use a valid executable
        .prefix(prefix)
        .build()
        .unwrap()
}

#[test]
fn test_eval_installed_formulae_with_empty_receipt() {
    let temp_dir = TempDir::new().unwrap();
    let opt_dir = temp_dir.path().join("opt");
    fs::create_dir_all(&opt_dir).unwrap();

    // Create a formula directory with empty INSTALL_RECEIPT.json
    let formula_dir = opt_dir.join("test-formula");
    fs::create_dir_all(&formula_dir).unwrap();
    let receipt_path = formula_dir.join("INSTALL_RECEIPT.json");
    fs::write(&receipt_path, "").unwrap(); // Empty file

    let brew = create_test_brew_with_prefix(temp_dir.path().to_path_buf());
    
    // Should not panic, should skip the empty receipt
    let result = brew.eval_installed_formulae_receipts();
    assert!(result.is_ok());
    let receipts = result.unwrap();
    assert!(!receipts.contains_key("test-formula"));
}

#[test]
fn test_eval_installed_formulae_with_corrupted_receipt() {
    let temp_dir = TempDir::new().unwrap();
    let opt_dir = temp_dir.path().join("opt");
    fs::create_dir_all(&opt_dir).unwrap();

    // Create a formula directory with corrupted JSON
    let formula_dir = opt_dir.join("test-formula");
    fs::create_dir_all(&formula_dir).unwrap();
    let receipt_path = formula_dir.join("INSTALL_RECEIPT.json");
    fs::write(&receipt_path, "{corrupted json").unwrap();

    let brew = create_test_brew_with_prefix(temp_dir.path().to_path_buf());
    
    // Should not panic, should skip the corrupted receipt
    let result = brew.eval_installed_formulae_receipts();
    assert!(result.is_ok());
    let receipts = result.unwrap();
    assert!(!receipts.contains_key("test-formula"));
}

#[test]
fn test_eval_installed_formulae_with_missing_receipt() {
    let temp_dir = TempDir::new().unwrap();
    let opt_dir = temp_dir.path().join("opt");
    fs::create_dir_all(&opt_dir).unwrap();

    // Create a formula directory without INSTALL_RECEIPT.json
    let formula_dir = opt_dir.join("test-formula");
    fs::create_dir_all(&formula_dir).unwrap();
    // No receipt file created

    let brew = create_test_brew_with_prefix(temp_dir.path().to_path_buf());
    
    // Should not panic, should skip the missing receipt
    let result = brew.eval_installed_formulae_receipts();
    assert!(result.is_ok());
    let receipts = result.unwrap();
    assert!(!receipts.contains_key("test-formula"));
}

#[test]
fn test_eval_installed_formulae_with_valid_receipt() {
    let temp_dir = TempDir::new().unwrap();
    let opt_dir = temp_dir.path().join("opt");
    fs::create_dir_all(&opt_dir).unwrap();

    // Create a formula directory with valid INSTALL_RECEIPT.json
    let formula_dir = opt_dir.join("test-formula");
    fs::create_dir_all(&formula_dir).unwrap();
    let receipt_path = formula_dir.join("INSTALL_RECEIPT.json");
    
    let valid_receipt = r#"{
        "source": {
            "spec": "stable",
            "versions": {
                "stable": "1.0.0",
                "head": null
            }
        },
        "installed_as_dependency": false,
        "installed_on_request": true
    }"#;
    
    fs::write(&receipt_path, valid_receipt).unwrap();

    let brew = create_test_brew_with_prefix(temp_dir.path().to_path_buf());
    
    // Should successfully parse the valid receipt
    let result = brew.eval_installed_formulae_receipts();
    assert!(result.is_ok());
    let receipts = result.unwrap();
    assert!(receipts.contains_key("test-formula"));
    assert!(receipts.get("test-formula").unwrap().installed_on_request);
}

#[test]
fn test_eval_installed_formulae_with_mixed_receipts() {
    let temp_dir = TempDir::new().unwrap();
    let opt_dir = temp_dir.path().join("opt");
    fs::create_dir_all(&opt_dir).unwrap();

    // Create multiple formula directories with different receipt states
    
    // Valid receipt
    let formula1 = opt_dir.join("valid-formula");
    fs::create_dir_all(&formula1).unwrap();
    fs::write(
        formula1.join("INSTALL_RECEIPT.json"),
        r#"{"source":{"spec":"stable","versions":{"stable":"1.0.0","head":null}},"installed_as_dependency":false,"installed_on_request":true}"#
    ).unwrap();

    // Empty receipt
    let formula2 = opt_dir.join("empty-formula");
    fs::create_dir_all(&formula2).unwrap();
    fs::write(formula2.join("INSTALL_RECEIPT.json"), "").unwrap();

    // Corrupted receipt
    let formula3 = opt_dir.join("corrupted-formula");
    fs::create_dir_all(&formula3).unwrap();
    fs::write(formula3.join("INSTALL_RECEIPT.json"), "{bad json}").unwrap();

    // Missing receipt
    let formula4 = opt_dir.join("missing-formula");
    fs::create_dir_all(&formula4).unwrap();

    let brew = create_test_brew_with_prefix(temp_dir.path().to_path_buf());
    
    // Should successfully parse only the valid receipt
    let result = brew.eval_installed_formulae_receipts();
    assert!(result.is_ok());
    let receipts = result.unwrap();
    
    assert_eq!(receipts.len(), 1);
    assert!(receipts.contains_key("valid-formula"));
    assert!(!receipts.contains_key("empty-formula"));
    assert!(!receipts.contains_key("corrupted-formula"));
    assert!(!receipts.contains_key("missing-formula"));
}

#[test]
fn test_eval_installed_formulae_skips_dotfiles() {
    let temp_dir = TempDir::new().unwrap();
    let opt_dir = temp_dir.path().join("opt");
    fs::create_dir_all(&opt_dir).unwrap();

    // Create a hidden directory (dotfile)
    let dotfile_dir = opt_dir.join(".hidden");
    fs::create_dir_all(&dotfile_dir).unwrap();
    fs::write(
        dotfile_dir.join("INSTALL_RECEIPT.json"),
        r#"{"source":{"spec":"stable","versions":{"stable":"1.0.0","head":null}},"installed_as_dependency":false,"installed_on_request":true}"#
    ).unwrap();

    let brew = create_test_brew_with_prefix(temp_dir.path().to_path_buf());
    
    // Should skip dotfiles
    let result = brew.eval_installed_formulae_receipts();
    assert!(result.is_ok());
    let receipts = result.unwrap();
    assert!(!receipts.contains_key(".hidden"));
}
