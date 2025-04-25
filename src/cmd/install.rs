use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use semver::Version;

use crate::package::Package;

#[derive(Debug, Parser, Default)]
#[clap(about = "Installs a package", alias = "i")]
pub struct Install {
    #[arg(
        help = "[branch] installs the latest version of that branch for core, lang and explorer\n[package] [branch_or_version] installs the latest branch of package or the specific version"
    )]
    pub args: Option<Vec<String>>,

    #[arg(long, help = "The architecture to install GreyCat for")]
    pub arch: Option<String>,

    #[arg(
        long,
        help = "The installation directory, defaults to $GREYCAT_HOME or $HOME/.greycat"
    )]
    pub dir: Option<PathBuf>,
}

impl Install {
    pub fn run(self) -> Result<()> {
        let dir = self.dir.unwrap_or_else(|| {
            std::env::var("GREYCAT_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    let mut home_dir = home::home_dir().unwrap_or_else(|| "/".into());
                    home_dir.push(".greycat");
                    home_dir
                })
        });

        // clean up previous directories to prevent ghost files
        fs::remove_dir_all(dir.join("bin")).ok();
        fs::remove_dir_all(dir.join("lib")).ok();
        fs::remove_dir_all(dir.join("include")).ok();
        fs::remove_dir_all(dir.join("misc")).ok();

        match self.args.as_deref() {
            Some([branch]) => {
                let arch = self.arch.or_else(|| Some(get_arch()));

                let core = Package::new("core", arch, branch);
                eprint!("installing {core}...");
                core.install_latest(&dir)?;

                let lang = Package::new("lang", None, branch);
                eprint!("installing {lang}...");
                lang.install_latest(&dir).ok();

                let explorer = Package::new("explorer", Some("noarch".to_string()), branch);
                eprint!("installing {explorer}...");
                explorer.install_latest(&dir).ok();
            }
            Some([name, branch_or_version]) => match Version::parse(branch_or_version) {
                Ok(version) => {
                    let arch = if name == "core" {
                        self.arch.or_else(|| Some(get_arch()))
                    } else {
                        None
                    };
                    let pkg = Package::new(name, arch, version.pre.as_str());

                    eprint!("installing {pkg}...");
                    pkg.install(version.clone(), &dir)?;
                }
                Err(_) => {
                    let arch = if name == "core" {
                        self.arch.or_else(|| Some(get_arch()))
                    } else {
                        None
                    };
                    let pkg = Package::new(name, arch, branch_or_version);

                    eprint!("installing {pkg}...");
                    pkg.install_latest(&dir)?;
                }
            },
            Some(_) => anyhow::bail!(
                "too many arguments, expected either: <branch> or <name> <branch_or_version>"
            ),
            None => {
                let arch = self.arch.or_else(|| Some(get_arch()));

                let core = Package::new("core", arch, "stable");
                let lang = Package::new("lang", None, "stable");
                let explorer = Package::new("explorer", Some("noarch".to_string()), "stable");

                eprint!("installing {core}...");
                core.install_latest(&dir)?;
                eprint!("installing {lang}...");
                lang.install_latest(&dir).ok();
                eprint!("installing {explorer}...");
                explorer.install_latest(&dir).ok();
            }
        }

        #[cfg(not(target_os = "windows"))]
        for entry in fs::read_dir(dir.join("bin"))? {
            let entry = entry?;
            let filepath = entry.path();
            if filepath.is_file() {
                let mut perm = fs::metadata(&filepath)?.permissions();
                std::os::unix::fs::PermissionsExt::set_mode(&mut perm, 0o755);
                fs::set_permissions(&filepath, perm)?;
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

#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
fn get_arch() -> String {
    "arm64-apple".to_owned()
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
fn get_arch() -> String {
    "x64-linux".to_owned()
}
