mod package;
mod registry;

use anyhow::Result;
use clap::{Parser, Subcommand};

use package::*;
use registry::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap()]
    branch: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[clap(about = "List a package branches if no [BRANCH] specified, otherwise list the versions")]
    List {
        #[arg(help = "The package name")]
        name: String,
        #[arg(help = "The package branch")]
        branch: Option<String>,
        #[arg(long, help = "Limit the number of version displayed")]
        limit: Option<usize>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::List {
            name,
            branch,
            limit,
        }) => match branch {
            Some(branch) => {
                let versions = list_package_versions(&name, &branch, limit)?;
                for version in versions {
                    println!("{version}");
                }
                Ok(())
            }
            None => {
                let branches: Vec<File> =
                    ureq::get(&format!("https://get.greycat.io/files/{name}/"))
                        .call()?
                        .into_json()?;
                for branch in branches {
                    let (_, branch) = branch.path.split_once('/').unwrap();
                    println!("{}", &branch[..branch.len() - 1]);
                }
                Ok(())
            }
        },
        None => {
            let branch = cli.branch.unwrap_or_else(|| "stable".to_string());
            Package::new("core", Some("x64-linux"), &branch).install()?;
            Package::new("lang", None, &branch).install().ok();
            Package::new("apps/explorer", None, &branch).install().ok();
            Ok(())
        }
    }
}
