use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Config;
use crate::self_update;

#[derive(Debug, Default, Serialize, Deserialize)]
struct UpdateCheckCache {
    last_checked: u64,
    latest_version: String,
}

pub fn check_and_notify(config: &Config) {
    let cache = load_cache();
    check_and_notify_internal(
        config.update_check_interval_hours,
        cache,
        &|| self_update::fetch_latest_tag(),
        &save_cache,
    );
}

fn check_and_notify_internal(
    interval_hours: u64,
    mut cache: UpdateCheckCache,
    fetcher: &dyn Fn() -> Result<String>,
    saver: &dyn Fn(&UpdateCheckCache),
) -> bool {
    if interval_hours == 0 {
        return false;
    }

    let elapsed = elapsed_hours(cache.last_checked);

    if elapsed >= interval_hours
        && let Ok(latest) = fetcher()
    {
        let latest_clean = latest.trim_start_matches('v').to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        cache.last_checked = now;
        cache.latest_version = latest_clean;
        saver(&cache);
    }

    let current = self_update::current_version();
    let latest_clean = cache.latest_version.trim_start_matches('v');
    if !latest_clean.is_empty()
        && self_update::version_compare(latest_clean, current) == std::cmp::Ordering::Greater
    {
        print_update_notice(current, latest_clean);
        return true;
    }

    false
}

fn print_update_notice(current: &str, latest: &str) {
    let msg1 = format!("  Update available: {} \u{2192} {}  ", current, latest);
    let msg2 = "  Run tmuxido --update to install.  ";
    let w1 = msg1.chars().count();
    let w2 = msg2.chars().count();
    let width = w1.max(w2);
    let border = "\u{2500}".repeat(width);
    println!("\u{250c}{}\u{2510}", border);
    println!("\u{2502}{}\u{2502}", pad_to_chars(&msg1, width));
    println!("\u{2502}{}\u{2502}", pad_to_chars(msg2, width));
    println!("\u{2514}{}\u{2518}", border);
}

fn pad_to_chars(s: &str, width: usize) -> String {
    let char_count = s.chars().count();
    if char_count >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - char_count))
    }
}

fn cache_path() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?
        .join("tmuxido");
    Ok(cache_dir.join("update_check.json"))
}

fn load_cache() -> UpdateCheckCache {
    cache_path()
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_cache(cache: &UpdateCheckCache) {
    if let Ok(path) = cache_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string(cache) {
            let _ = std::fs::write(path, json);
        }
    }
}

fn elapsed_hours(ts: u64) -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    now.saturating_sub(ts) / 3600
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn make_cache(last_checked: u64, latest_version: &str) -> UpdateCheckCache {
        UpdateCheckCache {
            last_checked,
            latest_version: latest_version.to_string(),
        }
    }

    fn now_ts() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    #[test]
    fn should_not_notify_when_interval_is_zero() {
        let cache = make_cache(0, "99.0.0");
        let fetcher_called = RefCell::new(false);

        let result = check_and_notify_internal(
            0,
            cache,
            &|| {
                *fetcher_called.borrow_mut() = true;
                Ok("99.0.0".to_string())
            },
            &|_| {},
        );

        assert!(!result);
        assert!(!fetcher_called.into_inner());
    }

    #[test]
    fn should_not_check_when_interval_not_elapsed() {
        let cache = make_cache(now_ts(), "");
        let fetcher_called = RefCell::new(false);

        check_and_notify_internal(
            24,
            cache,
            &|| {
                *fetcher_called.borrow_mut() = true;
                Ok("99.0.0".to_string())
            },
            &|_| {},
        );

        assert!(!fetcher_called.into_inner());
    }

    #[test]
    fn should_check_when_interval_elapsed() {
        let cache = make_cache(0, "");
        let fetcher_called = RefCell::new(false);

        check_and_notify_internal(
            1,
            cache,
            &|| {
                *fetcher_called.borrow_mut() = true;
                Ok(self_update::current_version().to_string())
            },
            &|_| {},
        );

        assert!(fetcher_called.into_inner());
    }

    #[test]
    fn should_not_notify_when_versions_equal() {
        let current = self_update::current_version();
        let cache = make_cache(now_ts(), current);

        let result = check_and_notify_internal(24, cache, &|| unreachable!(), &|_| {});

        assert!(!result);
    }

    #[test]
    fn should_detect_update_available() {
        let cache = make_cache(now_ts(), "99.0.0");

        let result = check_and_notify_internal(24, cache, &|| unreachable!(), &|_| {});

        assert!(result);
    }

    #[test]
    fn should_not_detect_update_when_current_is_newer() {
        let cache = make_cache(now_ts(), "0.0.1");

        let result = check_and_notify_internal(24, cache, &|| unreachable!(), &|_| {});

        assert!(!result);
    }
}
