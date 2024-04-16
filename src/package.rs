use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

use anyhow::{bail, Context, Result};
use zip::{
    read::ZipFile,
    result::{ZipError, ZipResult},
    unstable::stream::{ZipStreamFileMetadata, ZipStreamReader, ZipStreamVisitor},
};

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub arch: Option<String>,
    pub branch: String,
}

impl Package {
    pub fn new(name: &str, arch: Option<String>, branch: &str) -> Self {
        Self {
            name: name.to_owned(),
            arch,
            branch: branch.to_owned(),
        }
    }

    pub fn install(
        &self,
        version: semver::Version,
        install_dir: Option<&Path>,
    ) -> Result<Option<Version>> {
        let g_version = Version {
            major_minor: format!("{}.{}", version.major, version.minor),
            version: version.to_string(),
        };

        match self.download(&g_version) {
            Ok(archive) => {
                let install_dir = match install_dir {
                    Some(install_dir) => install_dir.to_owned(),
                    None => {
                        let mut home_dir = home::home_dir().unwrap_or_else(|| "/".into());
                        home_dir.push(".greycat");
                        home_dir
                    }
                };

                let archive = SmartZipExtractor { reader: archive };
                archive
                    .smart_extract(install_dir)
                    .context("extracting package content")?;

                eprintln!("{g_version}");
                Ok(Some(g_version))
            }
            Err(_) => Ok(None),
        }
    }

    pub fn install_latest(&self, install_dir: Option<&Path>) -> Result<Option<Version>> {
        use std::io::Write;
        use termcolor::{StandardStream, WriteColor, ColorChoice, ColorSpec, Color};

        let latest = match self.latest() {
            Ok(latest) => latest,
            Err(_) => {
                let mut stderr = StandardStream::stderr(ColorChoice::AlwaysAnsi);
                stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
                writeln!(&mut stderr, "not found")?;
                stderr.reset()?;
                return Ok(None);
            }
        };
        let version = semver::Version::parse(&latest.version)?;
        match self.install(version, install_dir)? {
            Some(version) => Ok(Some(version)),
            None => {
                let mut stderr = StandardStream::stderr(ColorChoice::AlwaysAnsi);
                stderr.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
                writeln!(&mut stderr, "not found")?;
                stderr.reset()?;
                Ok(None)
            }
        }
    }

    pub fn latest(&self) -> Result<Version> {
        let url = format!(
            "https://get.greycat.io/files/{}/{}/latest",
            self.name, self.branch
        );
        let latest: String = ureq::get(&url).call()?.into_string()?;
        let latest = Version::try_from(latest)?;
        Ok(latest)
    }

    pub fn download(
        &self,
        version: &Version,
    ) -> Result<ZipStreamReader<Box<dyn std::io::Read + std::marker::Send + std::marker::Sync>>>
    {
        let filepath = match &self.arch {
            Some(arch) => format!(
                "{name}/{branch}/{major_minor}/{arch}/{version}.zip",
                name = self.name,
                branch = self.branch,
                major_minor = version.major_minor,
                version = version.version
            ),
            None => format!(
                "{name}/{branch}/{major_minor}/{version}.zip",
                name = self.name,
                branch = self.branch,
                major_minor = version.major_minor,
                version = version.version
            ),
        };

        let url = format!("https://get.greycat.io/files/{filepath}");
        let res = ureq::get(&url).call()?;
        if res.status() != 200 {
            bail!("unable to download {filepath}")
        }

        Ok(ZipStreamReader::new(res.into_reader()))
    }
}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name, self.branch)
    }
}

#[derive(Debug)]
pub struct Version {
    pub major_minor: String,
    pub version: String,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.major_minor, self.version)
    }
}

impl TryFrom<String> for Version {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::prelude::v1::Result<Self, Self::Error> {
        match value.split_once('/') {
            Some((major_minor, version)) => Ok(Version {
                major_minor: major_minor.to_owned(),
                version: version.to_owned(),
            }),
            None => bail!("invalid version \"{value}\""),
        }
    }
}

struct SmartZipExtractor<R> {
    reader: ZipStreamReader<R>,
}

impl<R: io::Read> SmartZipExtractor<R> {
    fn smart_extract<P: AsRef<Path>>(self, directory: P) -> ZipResult<()> {
        struct Extractor<'a>(&'a Path);
        impl ZipStreamVisitor for Extractor<'_> {
            fn visit_file(&mut self, file: &mut ZipFile<'_>) -> ZipResult<()> {
                let filepath = file
                    .enclosed_name()
                    .ok_or(ZipError::InvalidArchive("Invalid file path"))?;

                let outpath = self.0.join(filepath);

                if file.name().ends_with('/') {
                    fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(p) = outpath.parent() {
                        fs::create_dir_all(p)?;
                    }
                    match fs::File::create(&outpath) {
                        Ok(mut outfile) => {
                            io::copy(file, &mut outfile)?;
                        }
                        Err(err) if err.kind() == ErrorKind::ExecutableFileBusy => {
                            // if we have greycat running, we need to be smarter
                            // lets read the permissions of the current executable file
                            let permissions = fs::File::open(&outpath)?.metadata()?.permissions();
                            // remove the current executable file
                            fs::remove_file(&outpath)?;
                            // re-create it
                            let mut outfile = fs::File::create(&outpath)?;
                            // set the same permissions back
                            outfile.set_permissions(permissions)?;
                            // and then copy the content of the zipfile to the new executable file output
                            io::copy(file, &mut outfile)?;
                        }
                        Err(err) => return Err(ZipError::Io(err)),
                    }
                }

                Ok(())
            }

            #[allow(unused)]
            fn visit_additional_metadata(
                &mut self,
                metadata: &ZipStreamFileMetadata,
            ) -> ZipResult<()> {
                #[cfg(unix)]
                {
                    let filepath = metadata
                        .enclosed_name()
                        .ok_or(ZipError::InvalidArchive("Invalid file path"))?;

                    let outpath = self.0.join(filepath);

                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = metadata.unix_mode() {
                        fs::set_permissions(outpath, fs::Permissions::from_mode(mode))?;
                    }
                }

                Ok(())
            }
        }

        self.visit(&mut Extractor(directory.as_ref()))
    }

    #[inline]
    pub fn visit<V: ZipStreamVisitor>(self, visitor: &mut V) -> ZipResult<()> {
        self.reader.visit(visitor)
    }
}
