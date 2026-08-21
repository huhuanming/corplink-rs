use std::cmp::Ordering;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;

const NPM_LATEST_URL: &str = "https://registry.npmjs.org/feilian-cli/latest";
const UPDATE_COMMAND: &str = "npm install --global feilian-cli@latest";

#[derive(Debug, Deserialize)]
struct NpmPackage {
    version: String,
}

#[derive(Debug, PartialEq)]
pub enum UpdateStatus {
    Available { current: String, latest: String },
    Current { current: String },
}

pub async fn check() -> Result<UpdateStatus> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .user_agent(format!("feilian-cli/{current}"))
        .build()
        .context("failed to create update client")?;
    let latest = client
        .get(NPM_LATEST_URL)
        .send()
        .await
        .context("failed to query npm")?
        .error_for_status()
        .context("npm returned an error while checking for updates")?
        .json::<NpmPackage>()
        .await
        .context("failed to parse npm update response")?
        .version;

    if compare_versions(&latest, &current) == Ordering::Greater {
        Ok(UpdateStatus::Available { current, latest })
    } else {
        Ok(UpdateStatus::Current { current })
    }
}

pub fn report(status: &UpdateStatus) {
    match status {
        UpdateStatus::Available { current, latest } => {
            eprintln!("feilian-cli update available: {current} -> {latest}");
            eprintln!("run: {UPDATE_COMMAND}");
        }
        UpdateStatus::Current { current } => {
            println!("feilian-cli {current} is up to date");
        }
    }
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    let left = version_parts(left);
    let right = version_parts(right);
    let length = left.len().max(right.len());

    for index in 0..length {
        let ordering = left
            .get(index)
            .copied()
            .unwrap_or_default()
            .cmp(&right.get(index).copied().unwrap_or_default());
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    Ordering::Equal
}

fn version_parts(version: &str) -> Vec<u64> {
    version
        .trim_start_matches('v')
        .split(['.', '-', '+'])
        .take_while(|part| part.chars().all(|character| character.is_ascii_digit()))
        .map(|part| part.parse().unwrap_or_default())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_release_versions() {
        assert_eq!(compare_versions("0.2.0", "0.1.9"), Ordering::Greater);
        assert_eq!(compare_versions("v1.2.0", "1.2"), Ordering::Equal);
        assert_eq!(compare_versions("1.2.3", "1.10.0"), Ordering::Less);
        assert_eq!(compare_versions("1.2.3-beta.1", "1.2.3"), Ordering::Equal);
    }
}
