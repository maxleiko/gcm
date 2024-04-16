use crate::registry::*;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about = "Lists a package branches and/or versions\neg. gcm list core, gcm list sdk/web testing", alias = "l")]
pub struct List {
    #[arg(help = "The package name")]
    name: String,

    #[arg(help = "The package branch")]
    branch: Option<String>,

    #[arg(long, help = "Limit the number of version displayed")]
    limit: Option<usize>,
}

impl List {
    pub fn run(self) -> Result<()> {
        let registry = Registry::default();
        match self.branch {
            Some(branch) => {
                let versions = registry.list_package_versions(&self.name, &branch, self.limit)?;
                for version in versions {
                    println!("{version}");
                }
                Ok(())
            }
            None => {
                let branches = registry.list_package_branches(&self.name)?;
                for branch in branches {
                    let (_, branch) = branch.path[..branch.path.len() - 1]
                        .rsplit_once('/')
                        .unwrap();
                    println!("{branch}");
                }
                Ok(())
            }
        }
    }
}
