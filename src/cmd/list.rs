use crate::registry::*;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(
    about = "Lists a package branches and/or versions\neg. gcm list core, gcm list sdk/web testing",
    alias = "l"
)]
pub struct List {
    #[arg(help = "The package name")]
    package: Option<String>,

    #[arg(help = "The package branch")]
    branch: Option<String>,

    #[arg(long, help = "Limit the number of version displayed", default_value = "5")]
    limit: usize,
}

impl List {
    pub fn run(self) -> Result<()> {
        let registry = Registry::default();
        match (self.package, self.branch) {
            (None, None) => {
                for package in registry.list_packages()? {
                    let name = package.path.strip_suffix('/').unwrap_or(&package.path);
                    println!("{name}");
                }
                Ok(())
            },
            (None, Some(branch)) => {
                eprintln!("TODO list all packages of a specific branch {branch}");
                Ok(())
            },
            (Some(package), None) => {
                let branches = registry.list_package_branches(&package)?;
                for branch in branches {
                    let (_, branch) = branch.path[..branch.path.len() - 1]
                        .rsplit_once('/')
                        .unwrap();
                    println!("{branch}");
                }
                Ok(())
            }
            (Some(package), Some(branch)) => {
                let versions =
                    registry.list_package_versions(&package, &branch, Some(self.limit))?;
                for version in versions {
                    println!("{version}");
                }
                Ok(())
            }
        }
    }
}
