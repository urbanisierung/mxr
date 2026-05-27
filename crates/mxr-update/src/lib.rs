use anyhow::{Context, Result};
use semver::Version;
use serde::Deserialize;
use std::io::Write;
use std::time::Duration;

#[derive(Debug)]
pub struct UpdateInfo {
    pub current: Version,
    pub latest: Version,
    pub tag_name: String,
    pub download_url: String,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[allow(unreachable_code)]
fn asset_name() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    {
        return "mxr-linux-x86_64";
    }
    #[cfg(target_arch = "aarch64")]
    {
        return "mxr-linux-aarch64";
    }
    panic!("unsupported architecture")
}

fn http_client() -> Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("mxr-updater")
        .build()
        .context("build http client")
}

pub fn check_for_update(repo: &str, current_version: &str) -> Result<Option<UpdateInfo>> {
    let current = Version::parse(current_version)
        .with_context(|| format!("parse version '{}'", current_version))?;
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let body = http_client()?
        .get(&url)
        .send()
        .context("fetch latest release")?
        .text()
        .context("read release body")?;
    let release: Release = serde_json::from_str(&body).context("parse release JSON")?;
    let tag_str = release.tag_name.trim_start_matches('v');
    let latest =
        Version::parse(tag_str).with_context(|| format!("parse latest version '{}'", tag_str))?;
    if latest <= current {
        return Ok(None);
    }
    let name = asset_name();
    let download_url = release
        .assets
        .iter()
        .find(|a| a.name == name)
        .map(|a| a.browser_download_url.clone())
        .ok_or_else(|| anyhow::anyhow!("no asset '{}' in release {}", name, release.tag_name))?;
    Ok(Some(UpdateInfo {
        current,
        latest,
        tag_name: release.tag_name,
        download_url,
    }))
}

pub fn perform_update(url: &str) -> Result<()> {
    let bytes = http_client()?
        .get(url)
        .send()
        .context("download binary")?
        .bytes()
        .context("read binary")?;
    let mut tmp = tempfile::NamedTempFile::new().context("create temp file")?;
    tmp.write_all(&bytes).context("write temp file")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(tmp.path(), std::fs::Permissions::from_mode(0o755))
            .context("chmod temp file")?;
    }
    self_replace::self_replace(tmp.path()).context("replace binary")?;
    Ok(())
}
