use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    #[cfg(feature = "gen")]
    gen::nginx_binding()?;

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

    pub fn nginx_binding() -> Result<()> {
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
            .allowlist_var("^(NGX|ngx)_.*$")
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
