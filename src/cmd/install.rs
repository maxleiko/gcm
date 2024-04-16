use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use semver::Version;

use crate::package::Package;

#[derive(Debug, Parser)]
#[clap(about = "Installs a package", alias = "i")]
pub struct Install {
    #[arg(
        help = "[branch] installs the latest version of that branch for core, lang and explorer\n[package] [branch_or_version] installs the latest branch of package or the specific version"
    )]
    args: Option<Vec<String>>,

    #[arg(long, help = "The architecture to install GreyCat for")]
    arch: Option<String>,

    #[arg(
        long,
        help = "By default install_dir=$HOME/.greycat/versions, this allows to override the path"
    )]
    install_dir: Option<PathBuf>,
}

impl Install {
    pub fn run(self) -> Result<()> {
        let install_dir = self.install_dir.as_deref();

        match self.args {
            Some(args) if args.len() == 1 => {
                let branch = &args[0];

                let arch = self.arch.or_else(|| Some(get_arch()));
                
                let core = Package::new("core", arch, branch);
                eprint!("installing {}...", core);
                core.install_latest(install_dir)?;

                let lang = Package::new("lang", None, branch);
                eprint!("installing {}...", lang);
                lang.install_latest(install_dir).ok();
                
                let explorer = Package::new("apps/explorer", None, branch);
                eprint!("installing {}...", explorer);
                explorer.install_latest(install_dir).ok();
            }
            Some(mut args) if args.len() == 2 => {
                let branch = args.pop().unwrap();
                let name = args.pop().unwrap();

                match Version::parse(&branch) {
                    Ok(version) => {
                        let arch = if name == "core" {
                            self.arch.or_else(|| Some(get_arch()))
                        } else {
                            None
                        };
                        let pkg = Package::new(&name, arch, version.pre.as_str());

                        eprint!("installing {}...", pkg);
                        pkg.install(version.clone(), install_dir)?;
                    }
                    Err(_) => {
                        let arch = if name == "core" {
                            self.arch.or_else(|| Some(get_arch()))
                        } else {
                            None
                        };
                        let pkg = Package::new(&name, arch, &branch);

                        eprint!("installing {}...", pkg);
                        pkg.install_latest(install_dir)?;
                    }
                }
            }
            Some(_) => anyhow::bail!(
                "too many arguments, expected either: <branch> or <name> <branch_or_version>"
            ),
            None => {
                let arch = self.arch.or_else(|| Some(get_arch()));

                let core = Package::new("core", arch, "stable");
                let lang = Package::new("lang", None, "stable");
                let explorer = Package::new("apps/explorer", None, "stable");

                eprint!("installing {}...", core);
                core.install_latest(install_dir)?;
                eprint!("installing {}...", lang);
                lang.install_latest(install_dir).ok();
                eprint!("installing {}...", explorer);
                explorer.install_latest(install_dir).ok();
            }
        }

        Ok(())
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "windows"))]
fn get_arch() -> String {
    "x64-windows".to_owned()
}

#[cfg(all(target_arch = "x86_64", target_os = "macos"))]
fn get_arch() -> String {
    "x64-apple".to_owned()
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
fn get_arch() -> String {
    "x64-linux".to_owned()
}
