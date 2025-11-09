use std::process::Command;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

/// Get the brewer binary path
fn brewer_bin() -> PathBuf {
    // Use the installed brewer or build it
    if let Ok(path) = which::which("brewer") {
        path
    } else {
        PathBuf::from("target/release/brewer")
    }
}

/// Get a temporary test cache directory
fn test_cache_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("brewer_test_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Clean up test cache directory
fn cleanup_test_cache(dir: &PathBuf) {
    let _ = fs::remove_dir_all(dir);
}

/// Run brewer command and return (stdout, stderr, exit_code)
fn run_brewer(args: &[&str], cache_dir: &PathBuf) -> (String, String, i32) {
    let output = Command::new(brewer_bin())
        .args(args)
        .env("BREWER_CACHE_DIR", cache_dir.to_str().unwrap())
        .output()
        .expect("Failed to execute brewer");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    (stdout, stderr, exit_code)
}

#[test]
fn test_cache_status_no_cache() {
    let cache_dir = test_cache_dir();
    
    let (stdout, stderr, exit_code) = run_brewer(&["cache", "status"], &cache_dir);
    
    if exit_code != 0 {
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
    }
    
    assert_eq!(exit_code, 0, "cache status should succeed");
    assert!(stdout.contains("No cache"), "Should report no cache initially: {}", stdout);
    
    cleanup_test_cache(&cache_dir);
}

#[test]
fn test_cache_clear() {
    let cache_dir = test_cache_dir();
    
    let (stdout, _, exit_code) = run_brewer(&["cache", "clear"], &cache_dir);
    
    assert_eq!(exit_code, 0, "cache clear should succeed");
    assert!(stdout.contains("Done") || stdout.contains("cleared") || stdout.contains("Clearing"), 
        "Should confirm clear: {}", stdout);
    
    cleanup_test_cache(&cache_dir);
}

#[test]
#[ignore] // Expensive test - run with --ignored
fn test_cache_update_and_status() {
    let cache_dir = test_cache_dir();
    
    println!("\n🧪 Testing cache update (this will take 2-3 minutes)...\n");
    
    // Build cache
    let start = Instant::now();
    let (stdout, stderr, exit_code) = run_brewer(&["cache", "update"], &cache_dir);
    let duration = start.elapsed();
    
    if exit_code != 0 {
        eprintln!("STDOUT: {}", stdout);
        eprintln!("STDERR: {}", stderr);
    }
    
    assert_eq!(exit_code, 0, "cache update should succeed");
    assert!(stdout.contains("Cache updated") || stdout.contains("successfully"), 
        "Should confirm update: {}", stdout);
    
    println!("✅ Cache built in {:.1}s\n", duration.as_secs_f64());
    
    // Check status
    let (stdout, _, exit_code) = run_brewer(&["cache", "status"], &cache_dir);
    
    assert_eq!(exit_code, 0, "cache status should succeed");
    assert!(stdout.contains("Active") || stdout.contains("Size"), 
        "Should show active cache: {}", stdout);
    
    println!("✅ Cache status verified\n");
    
    cleanup_test_cache(&cache_dir);
}
