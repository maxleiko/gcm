use std::process::Command;

use anyhow::{Context, Result};
use clap::Parser;

use crate::Install;

#[derive(Debug, Parser)]
#[clap(
    about = "Updates the currently installed packages on the same branch\nIf no installation found, installs latest 'stable'",
    alias = "u"
)]
pub struct Update {}

impl Update {
    pub fn run(self) -> Result<()> {
        let output = Command::new("greycat")
            .arg("-vv")
            .output()
            .context("unable to run 'greycat -vv'")?;
        let buf =
            String::from_utf8(output.stdout).context("'greycat -vv' returned non-UTF8 data")?;

        if let Some((version, arch)) = buf.split_once(' ') {
            let arch = arch
                .strip_prefix('(')
                .unwrap_or(arch)
                .strip_suffix(")\n")
                .unwrap_or(arch);
            let version = semver::Version::parse(version)?;

            Install {
                arch: Some(arch.to_string()),
                args: Some(vec![version.pre.to_string()]),
                ..Default::default()
            }
            .run()
        } else {
            Install::default().run()
        }
    }
}
