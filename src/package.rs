use zip::unstable::stream::ZipStreamReader;
use anyhow::{bail, Result};


#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub arch: Option<&'static str>,
    pub branch: String,
}

impl Package {
    pub fn new(name: &str, arch: Option<&'static str>, branch: &str) -> Self {
        Self {
            name: name.to_owned(),
            arch,
            branch: branch.to_owned(),
        }
    }

    pub fn install(&self) -> Result<Version> {
        eprint!("installing {}@{}...", self.name, self.branch);
        let latest = match self.latest() {
            Ok(latest) => latest,
            Err(_) => {
                eprintln!("not found");
                bail!("unable to find {}@{}", self.name, self.branch);
            },
        };
        eprintln!("{}", latest.version);
        let archive = self.download(&latest)?;
        let home_dir = home::home_dir().unwrap_or_else(|| "/".into());
        let install_dir = home_dir.join(".greycat");
        archive.extract(install_dir)?;
        Ok(latest)
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
        let filepath = match self.arch {
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
