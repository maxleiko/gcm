use std::str::FromStr;

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use semver::Version;
use serde::{Deserialize, Serialize};

pub struct Registry {
    url: String,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            url: "https://get.greycat.io/files".to_string(),
        }
    }
}

impl Registry {
    pub fn list_package_versions(
        &self,
        name: &str,
        branch: &str,
        limit: Option<usize>,
    ) -> Result<Vec<PackageVersion>> {
        let entries: Vec<File> = ureq::get(&format!("{}/{name}/{branch}", self.url))
            .call()
            .with_context(|| format!("no version found for \"{name}/{branch}\""))?
            .into_json()?;

        let mut versions = Vec::default();

        if name == "core" {
            // 'core' is target-dependant, meaning we have to browse one more level to get
            // to the different versions
            for file in entries {
                if file.path.ends_with('/') {
                    let entries: Vec<File> = ureq::get(&format!("{}/{}", self.url, file.path))
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

    pub fn list_package_branches(&self, name: &str) -> Result<Vec<File>> {
        let branches = ureq::get(&format!("{}/{name}/", self.url))
            .call()?
            .into_json()?;
        Ok(branches)
    }

    pub fn list_packages(&self) -> Result<Vec<File>> {
        let files: Vec<File> = ureq::get(&format!("{}/", self.url)).call()?.into_json()?;

        let mut packages = Vec::default();
        for file in files {
            match file.path.as_str() {
                "core/" | "lang/" => packages.push(file),
                "deps/" => (), // ignore
                _ => {
                    let files: Vec<File> = ureq::get(&format!("{}/{}", self.url, file.path))
                        .call()?
                        .into_json()?;
                    packages.extend(files);
                }
            }
        }

        packages.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(packages)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
    #[serde(deserialize_with = "greycat_time_to_datetime_utc")]
    pub last_modification: DateTime<Local>,
    pub path: String,
}

fn greycat_time_to_datetime_utc<'de, D>(de: D) -> Result<DateTime<Local>, D::Error>
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
    let date =
        DateTime::from_timestamp_micros(micros).ok_or_else(|| D::Error::custom("invalid time"))?;
    Ok(date.with_timezone(&Local))
}

#[derive(Debug)]
pub struct PackageVersion {
    last_modified: DateTime<Local>,
    version: Version,
}

impl std::fmt::Display for PackageVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:20} {}",
            self.version,
            self.last_modified.format("%Y-%m-%d %H:%M:%s")
        )
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
