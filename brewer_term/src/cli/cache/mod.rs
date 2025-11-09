use clap::{Args, Subcommand};
use brewer_engine::Engine;

#[derive(Args)]
pub struct Cache {
    #[command(subcommand)]
    pub command: CacheCommands,
}

#[derive(Subcommand)]
pub enum CacheCommands {
    /// Show cache status and information
    Status,
    /// Update cache manually
    Update,
    /// Clear the cache
    Clear,
}

impl Cache {
    pub fn run(&self, mut engine: Engine) -> anyhow::Result<()> {
        match &self.command {
            CacheCommands::Status => status(&engine),
            CacheCommands::Update => self.update(&mut engine),
            CacheCommands::Clear => clear(&mut engine),
        }
    }

    fn update(&self, engine: &mut Engine) -> anyhow::Result<()> {
        use colored::Colorize;

        println!("{}", "📦 Updating cache...".cyan());
        println!("This may take 2-3 minutes depending on the number of taps.");
        println!();

        // Force cache update by fetching fresh data
        let start = std::time::Instant::now();
        
        // Fetch and cache
        let state = engine.cache_or_latest()?;
        
        let elapsed = start.elapsed();

        println!("{}", "✅ Cache updated successfully!".green());
        println!("   Time taken: {:.1}s", elapsed.as_secs_f64());
        println!("   Formulae: {}", state.formulae.all.len());
        println!("   Casks: {}", state.casks.all.len());
        println!("   Cache size: {:.1} MB", 
            engine.store().cache_size()? as f64 / 1_000_000.0);

        Ok(())
    }
}

fn status(engine: &Engine) -> anyhow::Result<()> {
    use colored::Colorize;

    println!("{}", "📊 Cache Status".bold().cyan());
    println!();

    // Check if cache exists
    match engine.store().get_cached_brew_json()? {
        Some(cached_data) => {
            println!("  {} {}", "Status:".bold(), "Active".green());
            
            // Cache size
            let size_mb = cached_data.len() as f64 / 1_000_000.0;
            println!("  {} {:.1} MB", "Size:".bold(), size_mb);

            // Last update time
            if let Some(last_update) = engine.store().last_update()? {
                // Use system time to calculate age
                let now = std::time::SystemTime::now();
                let last_update_sys = std::time::UNIX_EPOCH + std::time::Duration::from_secs(
                    last_update.and_utc().timestamp() as u64
                );
                
                if let Ok(duration) = now.duration_since(last_update_sys) {
                    let age_str = format_duration(duration.as_secs() as i64);
                    println!("  {} {} ({} ago)", 
                        "Last updated:".bold(), 
                        last_update.format("%Y-%m-%d %H:%M:%S"),
                        age_str.dimmed());
                } else {
                    println!("  {} {}", "Last updated:".bold(), last_update.format("%Y-%m-%d %H:%M:%S"));
                }

                // Check if expired
                if engine.cache_expired()? {
                    println!("  {} {}", "Expired:".bold(), "Yes (will refresh on next command)".yellow());
                } else {
                    println!("  {} {}", "Valid:".bold(), "Yes".green());
                    if engine.cache_duration().is_none() {
                        println!("  {} {}", "Expires:".bold(), "Never".green());
                    }
                }
            }

            // TTL configuration
            println!();
            println!("  {} Configuration", "📋".bold());
            if let Ok(val) = std::env::var("BREWER_CACHE_NEVER_EXPIRE") {
                if val == "1" || val.to_lowercase() == "true" {
                    println!("    {} Never (BREWER_CACHE_NEVER_EXPIRE=1)", "TTL:".bold());
                }
            } else if let Ok(hours) = std::env::var("BREWER_CACHE_TTL") {
                println!("    {} {} hours (BREWER_CACHE_TTL={})", "TTL:".bold(), hours, hours);
            } else {
                println!("    {} 24 hours (default)", "TTL:".bold());
            }

            // Get state info
            if let Ok(Some(state)) = engine.cache() {
                println!();
                println!("  {} Packages", "📦".bold());
                println!("    {} {}", "Formulae:".bold(), state.formulae.all.len());
                println!("    {} {} installed", "  ".bold(), state.formulae.installed.len());
                println!("    {} {}", "Casks:".bold(), state.casks.all.len());
                println!("    {} {} installed", "  ".bold(), state.casks.installed.len());
            }
        }
        None => {
            println!("  {} {}", "Status:".bold(), "No cache".yellow());
            println!("  {} Run any command to build cache", "Tip:".bold());
        }
    }

    println!();
    Ok(())
}

fn clear(engine: &mut Engine) -> anyhow::Result<()> {
    use colored::Colorize;

    print!("🗑️  Clearing cache... ");
    std::io::Write::flush(&mut std::io::stdout())?;

    engine.store_mut().clear_brew_cache()?;

    println!("{}", "Done!".green());
    println!("Cache will be rebuilt on next command.");

    Ok(())
}

fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        if mins > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}h", hours)
        }
    } else {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        if hours > 0 {
            format!("{}d {}h", days, hours)
        } else {
            format!("{}d", days)
        }
    }
}
