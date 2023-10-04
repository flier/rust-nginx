use std::env;

use std::path::{Path, PathBuf};

use anyhow::Result;
use cfg_if::cfg_if;
use tracing::info;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    if env::var("DOCS_RS").is_ok()
        || env::var("CARGO_CFG_DOC").is_ok()
        || cfg!(feature = "docsrs")
        || cfg!(feature = "cargo-clippy")
        || cfg!(target_family = "windows")
    {
        info!("skip building nginx for clippy and docs");

        return Ok(());
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(out_dir.as_str());

    cargo_emit::rerun_if_env_changed!("NGINX_SRC_FILE");

    let src_file = if let Ok(filename) = env::var("NGINX_SRC_FILE") {
        info!(filename, "using nginx source file");

        PathBuf::from(filename)
    } else {
        cfg_if! {
            if #[cfg(feature = "fetch")] {
                fetch::download_nginx_src(out_dir)?
            } else {
                anyhow::bail!("NGINX_SRC_FILE is not set")
            }
        }
    };

    let src_dir = extract::decompress_nginx_src(&src_file, out_dir)?;

    build::link_nginx_src(&src_dir, &out_dir.join("nginx"))?;

    let build_dir = out_dir.join("build");

    #[cfg(feature = "build")]
    build::compile_nginx_src(&src_dir, &build_dir, &out_dir.join("dist"))?;

    cargo_emit::rerun_if_env_changed!("STATIC_LINK_NGINX");

    if cfg!(feature = "static-lib") || env::var("STATIC_LINK_NGINX").is_ok() {
        build::link_static_lib(&build_dir)?;
    }

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
    pub fn download_nginx_src(dir: &Path) -> Result<PathBuf> {
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

            download_file(&src_url, &filename)?;
        }

        Ok(filename)
    }

    #[instrument]
    fn download_file(url: &Url, filename: &Path) -> Result<()> {
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

            let mut f = File::create(filename)?;

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
    use tracing::{info, instrument};

    #[instrument]
    pub fn decompress_nginx_src(file: &Path, to: &Path) -> Result<PathBuf> {
        let dir = to.join(
            file.file_name()
                .and_then(|s| s.to_str())
                .and_then(|s| s.strip_suffix(".tar.gz"))
                .ok_or_else(|| anyhow!("expect filename with `.tar.gz` ext"))?,
        );

        if dir.is_dir() {
            info!(path = ?dir, "use extracted source dir");
        } else {
            let f = File::open(file)?;
            let mut ar = tar::Archive::new(libflate::gzip::Decoder::new(f)?);

            ar.unpack(to)?;
        }

        Ok(dir)
    }
}

#[cfg(feature = "build")]
mod build {
    use std::fs;
    #[cfg(target_family = "unix")]
    use std::os::unix::fs::symlink;
    #[cfg(target_family = "windows")]
    use std::os::windows::fs::symlink_dir as symlink;
    use std::path::Path;
    use std::process::Command;

    use anyhow::Result;
    use ngx_build::CommandExt;
    use tracing::{info, instrument};

    #[instrument]
    pub fn link_nginx_src(original: &Path, link: &Path) -> Result<()> {
        if !link.is_symlink() {
            symlink(original, link)?;
        }

        Ok(())
    }

    const OPTIONAL_HTTP_MODULES: &[&str] = &[
        #[cfg(feature = "http_ssl")]
        "http_ssl",
        #[cfg(feature = "http_v2")]
        "http_v2",
        #[cfg(feature = "http_v3")]
        "http_v3",
        #[cfg(feature = "http_realip")]
        "http_realip",
        #[cfg(feature = "http_addition")]
        "http_addition",
        #[cfg(feature = "http_xslt")]
        "http_xslt",
        #[cfg(feature = "http_image_filter")]
        "http_image_filter",
        #[cfg(feature = "http_geoip")]
        "http_geoip",
        #[cfg(feature = "http_sub")]
        "http_sub",
        #[cfg(feature = "http_dav")]
        "http_dav",
        #[cfg(feature = "http_flv")]
        "http_flv",
        #[cfg(feature = "http_mp4")]
        "http_mp4",
        #[cfg(feature = "http_gunzip")]
        "http_gunzip",
        #[cfg(feature = "http_gzip_static")]
        "http_gzip_static",
        #[cfg(feature = "http_auth_request")]
        "http_auth_request",
        #[cfg(feature = "http_random_index")]
        "http_random_index",
        #[cfg(feature = "http_secure_link")]
        "http_secure_link",
        #[cfg(feature = "http_degradation")]
        "http_degradation",
        #[cfg(feature = "http_slice")]
        "http_slice",
        #[cfg(feature = "http_stub_status")]
        "http_stub_status",
    ];

    const OPTIONAL_MAIL_MODULES: &[&str] = &[
        #[cfg(feature = "mail_ssl")]
        "mail_ssl",
    ];

    const OPTIONAL_STREAM_MODULES: &[&str] = &[
        #[cfg(feature = "stream_ssl")]
        "stream_ssl",
        #[cfg(feature = "stream_realip")]
        "stream_realip",
        #[cfg(feature = "stream_geoip")]
        "stream_geoip",
        #[cfg(feature = "stream_ssl_preread")]
        "stream_ssl_preread",
    ];

    const BUILD_IN_HTTP_MODULES: &[&str] = &[
        #[cfg(not(feature = "http_charset"))]
        "http_charset",
        #[cfg(not(feature = "http_gzip"))]
        "http_gzip",
        #[cfg(not(feature = "http_ssi"))]
        "http_ssi",
        #[cfg(not(feature = "http_userid"))]
        "http_userid",
        #[cfg(not(feature = "http_access"))]
        "http_access",
        #[cfg(not(feature = "http_auth_basic"))]
        "http_auth_basic",
        #[cfg(not(feature = "http_mirror"))]
        "http_mirror",
        #[cfg(not(feature = "http_autoindex"))]
        "http_autoindex",
        #[cfg(not(feature = "http_geo"))]
        "http_geo",
        #[cfg(not(feature = "http_map"))]
        "http_map",
        #[cfg(not(feature = "http_split_clients"))]
        "http_split_clients",
        #[cfg(not(feature = "http_referer"))]
        "http_referer",
        #[cfg(not(feature = "http_rewrite"))]
        "http_rewrite",
        #[cfg(not(feature = "http_proxy"))]
        "http_proxy",
        #[cfg(not(feature = "http_fastcgi"))]
        "http_fastcgi",
        #[cfg(not(feature = "http_uwsgi"))]
        "http_uwsgi",
        #[cfg(not(feature = "http_scgi"))]
        "http_scgi",
        #[cfg(not(feature = "http_grpc"))]
        "http_grpc",
        #[cfg(not(feature = "http_memcached"))]
        "http_memcached",
        #[cfg(not(feature = "http_limit_conn"))]
        "http_limit_conn",
        #[cfg(not(feature = "http_limit_req"))]
        "http_limit_req",
        #[cfg(not(feature = "http_empty_gif"))]
        "http_empty_gif",
        #[cfg(not(feature = "http_browser"))]
        "http_browser",
        #[cfg(not(feature = "http_upstream_hash"))]
        "http_upstream_hash",
        #[cfg(not(feature = "http_upstream_ip_hash"))]
        "http_upstream_ip_hash",
        #[cfg(not(feature = "http_upstream_least_conn"))]
        "http_upstream_least_conn",
        #[cfg(not(feature = "http_upstream_random"))]
        "http_upstream_random",
        #[cfg(not(feature = "http_upstream_keepalive"))]
        "http_upstream_keepalive",
        #[cfg(not(feature = "http_upstream_zone"))]
        "http_upstream_zone",
    ];

    const BUILD_IN_MAIL_MODULES: &[&str] = &[
        #[cfg(not(feature = "mail_pop3"))]
        "mail_pop3",
        #[cfg(not(feature = "mail_imap"))]
        "mail_imap",
        #[cfg(not(feature = "mail_smtp"))]
        "mail_smtp",
    ];

    const BUILD_IN_STREAM_MODULES: &[&str] = &[
        #[cfg(not(feature = "stream_limit_conn"))]
        "stream_limit_conn",
        #[cfg(not(feature = "stream_access"))]
        "stream_access",
        #[cfg(not(feature = "stream_geo"))]
        "stream_geo",
        #[cfg(not(feature = "stream_map"))]
        "stream_map",
        #[cfg(not(feature = "stream_split_clients"))]
        "stream_split_clients",
        #[cfg(not(feature = "stream_return"))]
        "stream_return",
        #[cfg(not(feature = "stream_set"))]
        "stream_set",
        #[cfg(not(feature = "stream_upstream_hash"))]
        "stream_upstream_hash",
        #[cfg(not(feature = "stream_upstream_least_conn"))]
        "stream_upstream_least_conn",
        #[cfg(not(feature = "stream_upstream_random"))]
        "stream_upstream_random",
        #[cfg(not(feature = "stream_upstream_zone"))]
        "stream_upstream_zone",
    ];

    #[instrument]
    pub fn compile_nginx_src(src_dir: &Path, build_dir: &Path, dist_dir: &Path) -> Result<()> {
        if dist_dir.join("sbin/nginx").is_file() {
            info!(build = ?build_dir, dist = ?dist_dir, "use installed nginx");
        } else {
            let mut builder = ngx_build::Builder::default();

            if cfg!(feature = "debug-log") {
                builder.with_debug();
            }

            if cfg!(feature = "compat") {
                builder.with_compat();
            }

            if cfg!(feature = "threads") {
                builder.with_threads();
            }

            #[cfg(all(feature = "file_aio", any(target_os = "linux", target_os = "freebsd")))]
            builder.with_file_aio();

            builder
                .src_dir(src_dir)
                .build_dir(build_dir)
                .out_dir(dist_dir);

            if cfg!(feature = "http") {
                builder
                    .with_modules(OPTIONAL_HTTP_MODULES)
                    .without_modules(BUILD_IN_HTTP_MODULES);

                if cfg!(not(feature = "http_cache")) {
                    builder.without_http_cache();
                }
            } else {
                builder.without_http();
            }

            if cfg!(feature = "mail") {
                builder
                    .with_mail()
                    .with_modules(OPTIONAL_MAIL_MODULES)
                    .without_modules(BUILD_IN_MAIL_MODULES);
            }

            if cfg!(feature = "stream") {
                builder
                    .with_stream()
                    .with_modules(OPTIONAL_STREAM_MODULES)
                    .without_modules(BUILD_IN_STREAM_MODULES);
            }

            let configure: ngx_build::Configure = builder.configure()?;
            let make = configure.run()?;

            make.build()?;
            make.install()?;
        }

        Ok(())
    }

    pub fn link_static_lib(build_dir: &Path) -> Result<()> {
        let mut cc = cc::Build::new();

        for dir in &[
            "",
            "src/core",
            #[cfg(feature = "event")]
            "src/event",
            #[cfg(feature = "event")]
            "src/event/modules",
            #[cfg(feature = "http")]
            "src/http",
            #[cfg(feature = "http")]
            "src/http/v2",
            #[cfg(feature = "http")]
            "src/http/modules",
            #[cfg(feature = "mail")]
            "src/mail",
            #[cfg(target_family = "unix")]
            "src/os/unix",
            #[cfg(target_family = "windows")]
            "src/os/win32",
            #[cfg(feature = "stream")]
            "src/stream",
        ] {
            for entry in fs::read_dir(build_dir.join(dir))? {
                let entry = entry?;
                let filename = entry.path();

                if filename.is_file() && matches!(filename.extension(), Some(ext) if ext == "o") {
                    if matches!(filename.file_name(), Some(name) if name == "nginx.o") {
                        let new = filename.with_file_name("nginx-no-main.o");

                        Command::new("objcopy")
                            .args(&[
                                "--strip-symbol=main".to_owned(),
                                filename.display().to_string(),
                                new.display().to_string(),
                            ])
                            .current_dir(filename.parent().unwrap())
                            .run()?;

                        cc.object(new);
                    } else {
                        cc.object(filename);
                    }
                }
            }
        }

        cc.pic(true)
            .shared_flag(true)
            .static_flag(true)
            .out_dir(build_dir)
            .cargo_metadata(true)
            .compile("nginx");

        Ok(())
    }
}
