use std::env;

use anyhow::Result;
use tracing::info;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    if env::var("DOCS_RS").is_ok()
        || env::var("CARGO_CFG_DOC").is_ok()
        || cfg!(feature = "docsrs")
        || cfg!(feature = "cargo-clippy")
        || cfg!(target_family = "windows")
    {
        info!("skip gen binding for clippy and docs");

        return Ok(());
    }

    #[cfg(feature = "gen")]
    gen::generate_nginx_binding()?;

    #[cfg(feature = "static-link")]
    build::link_static_libraries();

    Ok(())
}

#[cfg(feature = "gen")]
mod gen {
    use std::{
        env,
        path::{Path, PathBuf},
    };

    use anyhow::Result;
    use cfg_if::cfg_if;
    use tracing::{debug, info};

    pub fn generate_nginx_binding() -> Result<()> {
        cargo_emit::rerun_if_env_changed!("NGINX_DIR");

        let (nginx_dir, build_dir) = if let Ok(dir) = env::var("NGINX_DIR") {
            debug!(dir, "using nginx source file");

            let dir = PathBuf::from(dir);

            (dir.clone(), dir)
        } else {
            cfg_if! {
                if #[cfg(feature = "vendored")] {
                    info!(src=?ngx_src::SRC_DIR, build=?ngx_src::BUILD_DIR, "using vendored nginx source");

                    (PathBuf::from(ngx_src::SRC_DIR.to_string()), PathBuf::from(ngx_src::BUILD_DIR.to_string()))
                } else {
                    anyhow::bail!("NGINX_DIR is not set")
                }
            }
        };

        cargo_emit::rerun_if_changed!("nginx.h");

        let out_dir = env::var("OUT_DIR").unwrap();

        let binding = ngx_build::binding::Binding {
            header: "nginx.h",
            src_dir: &nginx_dir.join("src"),
            build_dir: &build_dir,
            out_file: &Path::new(out_dir.as_str()).join("bindings.rs"),
            event: cfg!(feature = "event"),
            http: cfg!(feature = "http"),
            mail: cfg!(feature = "mail"),
            stream: cfg!(feature = "stream"),
        };

        binding.generate()?;

        Ok(())
    }
}

#[cfg(feature = "static-link")]
mod build {
    pub fn link_static_libraries() {
        let mut cfg = pkg_config::Config::new();

        if cfg.statik(true).probe("libssl").is_err() {
            cargo_emit::rustc_link_lib!("crypto", "ssl");
        }
        if cfg.statik(true).probe("libcrypt").is_err() {
            cargo_emit::rustc_link_lib!("crypt");
        }
        if cfg.statik(true).probe("libpcre2-8").is_err() {
            cargo_emit::rustc_link_lib!("pcre2-8");
        }
        if cfg.statik(true).probe("zlib").is_err() {
            cargo_emit::rustc_link_lib!("z");
        }

        cargo_emit::rustc_link_search!(ngx_src::BUILD_DIR);
        cargo_emit::rustc_link_lib!(
            "nginx" => "static"
        );
    }
}
