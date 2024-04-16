use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about = "Updates the currently installed packages", alias = "u")]
pub struct Update {}

impl Update {
    pub fn run(self) -> Result<()> {
        todo!()
    }
}
