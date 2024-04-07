use std::str::FromStr;

use anyhow::Context;
use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    #[serde(deserialize_with = "greycat_time_to_datetime_utc")]
    pub last_modification: DateTime<Utc>,
    pub path: String,
}

fn greycat_time_to_datetime_utc<'de, D>(de: D) -> Result<DateTime<Utc>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    struct GreyCatTime {
        epoch: i32,
        us: i32,
    }

    let time = GreyCatTime::deserialize(de)?;
    let micros = ((time.epoch as i64) * 1_000_000) + (time.us as i64);
    DateTime::<Utc>::from_timestamp_micros(micros).ok_or_else(|| D::Error::custom("invalid time"))
}

#[derive(Debug)]
pub struct PackageVersion {
    last_modified: DateTime<Utc>,
    version: Version,
}

impl std::fmt::Display for PackageVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:20} {}", self.version, self.last_modified)
    }
}

impl std::cmp::Eq for PackageVersion {}

impl std::cmp::PartialEq for PackageVersion {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
    }
}

impl std::cmp::Ord for PackageVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.version.cmp(&other.version)
    }
}

impl std::cmp::PartialOrd for PackageVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub fn list_package_versions(
    name: &str,
    branch: &str,
    limit: Option<usize>,
) -> anyhow::Result<Vec<PackageVersion>> {
    let entries: Vec<File> = ureq::get(&format!("https://get.greycat.io/files/{name}/{branch}"))
        .call()
        .with_context(|| format!("no version found for \"{name}/{branch}\""))?
        .into_json()?;

    let mut versions = Vec::default();

    if name == "core" {
        // 'core' is target-dependant, meaning we have to browse one more level to get
        // to the different versions
        for file in entries {
            if file.path.ends_with('/') {
                let entries: Vec<File> =
                    ureq::get(&format!("https://get.greycat.io/files/{}", file.path))
                        .call()
                        .with_context(|| format!("no version found for \"{name}/{branch}\""))?
                        .into_json()?;
                for file in entries {
                    if file.path.ends_with('/') {
                        add_entries(file, &mut versions)?;
                    }
                }
            }
        }
    } else {
        for file in entries {
            if file.path.ends_with('/') {
                add_entries(file, &mut versions)?;
            }
        }
    }

    versions.sort();
    versions.dedup();
    if let Some(limit) = limit {
        let len = versions.len();
        if len > limit {
            versions = versions.drain(versions.len() - limit..).collect();
        }
    }

    Ok(versions)
}

fn add_entries(file: File, versions: &mut Vec<PackageVersion>) -> anyhow::Result<()> {
    let entries: Vec<File> = ureq::get(&format!("https://get.greycat.io/files/{}", file.path))
        .call()?
        .into_json()?;

    for file in entries {
        if file.path.ends_with(".zip") {
            let (_, version) = file.path[..file.path.len() - 4].rsplit_once('/').unwrap();
            if let Ok(version) = Version::from_str(version) {
                versions.push(PackageVersion {
                    last_modified: file.last_modification,
                    version,
                });
            }
        }
    }

    Ok(())
}
