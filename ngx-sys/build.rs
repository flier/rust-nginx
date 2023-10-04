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

        let src_dir = nginx_dir.join("src");
        let mut bindings = bindgen::builder()
            .header("nginx.h")
            .clang_args(&[
                format!("-I{}", build_dir.display()),
                format!("-I{}", src_dir.join("core").display()),
                #[cfg(target_family = "unix")]
                format!("-I{}", src_dir.join("os/unix").display()),
                #[cfg(target_family = "windows")]
                format!("-I{}", src_dir.join("os/win32").display()),
            ])
            .allowlist_type("^(NGX|ngx)_.*$")
            .allowlist_function("^(NGX|ngx)_.*$")
            .allowlist_var("^(NGX|ngx|NGINX|nginx)_.*$")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks));

        if cfg!(feature = "event") {
            bindings = bindings.clang_args(&[
                "-DNGX_EVENT".to_string(),
                format!("-I{}", src_dir.join("event").display()),
            ]);
        }

        if cfg!(feature = "http") {
            bindings = bindings.clang_args(&[
                "-DNGX_HTTP".to_string(),
                format!("-I{}", src_dir.join("http").display()),
                format!("-I{}", src_dir.join("http/modules").display()),
                format!("-I{}", src_dir.join("http/v2").display()),
            ]);
        }

        if cfg!(feature = "mail") {
            bindings = bindings.clang_args(&[
                "-DNGX_MAIL".to_string(),
                format!("-I{}", src_dir.join("mail").display()),
            ]);
        }

        if cfg!(feature = "stream") {
            bindings = bindings.clang_args(&[
                "-DNGX_STREAM".to_string(),
                format!("-I{}", src_dir.join("stream").display()),
            ]);
        }

        let builder = bindings.generate()?;

        let out_dir = env::var("OUT_DIR").unwrap();
        let out_dir = Path::new(out_dir.as_str());

        builder.write_to_file(out_dir.join("bindings.rs"))?;

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
