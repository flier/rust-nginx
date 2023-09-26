use std::env;
#[cfg(target_family = "unix")]
use std::os::unix::fs::symlink;
#[cfg(target_family = "windows")]
use std::os::windows::fs::symlink_dir as symlink;
use std::path::{Path, PathBuf};

use anyhow::Result;
use cfg_if::cfg_if;
use tracing::{debug, info};

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(out_dir.as_str());

    cargo_emit::rerun_if_env_changed!("NGINX_SRC_FILE");

    let src_file = if let Ok(filename) = env::var("NGINX_SRC_FILE") {
        debug!(filename, "using nginx source file");

        PathBuf::from(filename)
    } else {
        cfg_if! {
            if #[cfg(feature = "fetch")] {
                fetch::nginx_src(&out_dir)?
            } else {
                anyhow::bail!("NGINX_SRC_FILE is not set")
            }
        }
    };

    debug!(dir = ?out_dir, "extracting nginx source file");

    let src_dir = extract::nginx_src(&src_file, &out_dir)?;

    let nginx_dir = out_dir.join("nginx");

    if !nginx_dir.is_symlink() {
        debug!(
            original = ?src_dir,
            link = ?nginx_dir,
            "symlink nginx source"
        );

        symlink(&src_dir, &nginx_dir)?;
    }

    info!(dir = ?src_dir, "building nginx source");

    #[cfg(feature = "build")]
    build::nginx(&src_dir, &out_dir)?;

    Ok(())
}

#[cfg(feature = "fetch")]
mod fetch {
    use std::env;
    use std::fs::File;
    use std::path::{Path, PathBuf};

    use anyhow::{anyhow, Result};
    use tracing::{debug, info, instrument};
    use url::Url;

    #[cfg(feature = "v1_22")]
    const NGINX_SRC_URL: &str = "https://nginx.org/download/nginx-1.22.1.tar.gz";
    #[cfg(feature = "v1_24")]
    const NGINX_SRC_URL: &str = "https://nginx.org/download/nginx-1.24.0.tar.gz";
    #[cfg(feature = "v1_25")]
    const NGINX_SRC_URL: &str = "https://nginx.org/download/nginx-1.25.2.tar.gz";

    #[instrument]
    pub fn nginx_src(dir: &Path) -> Result<PathBuf> {
        use std::borrow::Cow;

        cargo_emit::rerun_if_env_changed!("NGINX_SRC_URL");

        let src_url = if let Ok(url) = env::var("NGINX_SRC_URL") {
            debug!(url, from = "NGINX_SRC_URL", "use source file");

            Cow::from(url)
        } else {
            debug!(url = NGINX_SRC_URL, from = "feature", "use source file");

            Cow::from(NGINX_SRC_URL)
        };
        let src_url = Url::parse(&src_url)?;

        let path = Path::new(src_url.path());
        let filename = dir.join(path.file_name().unwrap());

        if filename.is_file() {
            info!(path = ?filename, "use prefetched source file");
        } else {
            info!(url = %src_url, "fetching source file");

            download(&src_url, &filename)?;
        }

        Ok(filename)
    }

    #[instrument]
    fn download(url: &Url, filename: &Path) -> Result<()> {
        let mut res = reqwest::blocking::Client::builder()
            .build()?
            .get(url.as_str())
            .send()?;

        if res.status().is_success() {
            debug!(
                path = ?filename,
                size = res.content_length().unwrap(),
                code = res.status().as_u16(),
                "saved source file");

            let mut f = File::create(&filename)?;

            res.copy_to(&mut f)?;

            Ok(())
        } else {
            Err(anyhow!(
                "failed to fetch nginx source file, status: {}",
                res.status()
            ))
        }
    }
}

mod extract {
    use std::fs::File;
    use std::path::{Path, PathBuf};

    use anyhow::{anyhow, Result};
    use tracing::instrument;

    #[instrument]
    pub fn nginx_src(file: &Path, to: &Path) -> Result<PathBuf> {
        let f = File::open(&file)?;
        let mut ar = tar::Archive::new(libflate::gzip::Decoder::new(f)?);

        ar.unpack(to)?;

        Ok(to.join(
            file.file_name()
                .and_then(|s| s.to_str())
                .and_then(|s| s.strip_suffix(".tar.gz"))
                .ok_or_else(|| anyhow!("expect filename with `.tar.gz` ext"))?,
        ))
    }
}

#[cfg(feature = "build")]
mod build {
    use std::path::Path;

    use anyhow::Result;
    use tracing::instrument;

    const MODULES: &[&str] = &[
        #[cfg(feature = "http_addition")]
        "http_addition",
        #[cfg(feature = "http_auth_request")]
        "http_auth_request",
        #[cfg(feature = "http_flv")]
        "http_flv",
        #[cfg(feature = "http_gunzip")]
        "http_gunzip",
        #[cfg(feature = "http_gzip_static")]
        "http_gzip_static",
        #[cfg(feature = "http_random_index")]
        "http_random_index",
        #[cfg(feature = "http_realip")]
        "http_realip",
        #[cfg(feature = "http_secure_link")]
        "http_secure_link",
        #[cfg(feature = "http_slice")]
        "http_slice",
        #[cfg(feature = "http_ssl")]
        "http_ssl",
        #[cfg(feature = "http_stub_status")]
        "http_stub_status",
        #[cfg(feature = "http_sub")]
        "http_sub",
        #[cfg(feature = "http_v2")]
        "http_v2",
        #[cfg(feature = "stream_realip")]
        "stream_realip",
        #[cfg(feature = "stream_ssl")]
        "stream_ssl",
        #[cfg(feature = "stream_ssl_preread")]
        "stream_ssl_preread",
        #[cfg(feature = "http_geoip")]
        "http_geoip",
        #[cfg(feature = "stream_geoip")]
        "stream_geoip",
        #[cfg(feature = "http_dav")]
        "http_dav",
        #[cfg(feature = "http_degradation")]
        "http_degradation",
        #[cfg(feature = "http_image_filter")]
        "http_image_filter",
        #[cfg(feature = "http_mp4")]
        "http_mp4",
        #[cfg(feature = "http_perl")]
        "http_perl",
        #[cfg(feature = "http_v3")]
        "http_v3",
        #[cfg(feature = "http_xslt")]
        "http_xslt",
    ];

    #[cfg(not(target_family = "windows"))]
    #[instrument]
    pub fn nginx(src_dir: &Path, out_dir: &Path) -> Result<()> {
        let mut builder = ngx_build::Builder::default();

        #[cfg(feature = "compat")]
        builder.with_compat();

        #[cfg(feature = "stream")]
        builder.with_stream();

        #[cfg(feature = "threads")]
        builder.with_threads();

        #[cfg(all(feature = "file_aio", any(target_os = "linux", target_os = "freebsd")))]
        builder.with_file_aio();

        let build_dir = out_dir.join(&"build");
        let dist_dir = out_dir.join("dist");

        builder
            .src_dir(&src_dir)
            .build_dir(&build_dir)
            .out_dir(&dist_dir)
            .with_modules(MODULES);

        builder.build()?;

        Ok(())
    }

    #[cfg(target_family = "windows")]
    #[instrument]
    pub fn nginx(src_dir: &Path, out_dir: &Path) -> Result<()> {
        use anyhow::anyhow;

        Err(anyhow!("windows is not supported"))
    }
}
