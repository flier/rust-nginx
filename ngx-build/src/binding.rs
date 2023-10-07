use std::path::Path;

use crate::Error;

pub struct Binding<'a> {
    pub header: &'a str,
    pub src_dir: &'a Path,
    pub build_dir: &'a Path,
    pub out_file: &'a Path,
    pub event: bool,
    pub http: bool,
    pub mail: bool,
    pub stream: bool,
}

impl<'a> Binding<'a> {
    pub fn generate(self) -> Result<(), Error> {
        let mut bindings = bindgen::builder()
            .header(self.header)
            .clang_args(
                [
                    self.build_dir,
                    &self.src_dir.join("core"),
                    #[cfg(target_family = "unix")]
                    &self.src_dir.join("os/unix"),
                    #[cfg(target_family = "windows")]
                    &self.src_dir.join("os/win32"),
                ]
                .into_iter()
                .map(|p| format!("-I{}", p.display()))
                .collect::<Vec<_>>(),
            )
            .allowlist_type("^(NGX|ngx)_.*$")
            .allowlist_function("^(NGX|ngx)_.*$")
            .allowlist_var("^(NGX|ngx|NGINX|nginx)_.*$")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks));

        if self.event {
            bindings = bindings.clang_args(&[
                "-DNGX_EVENT".to_string(),
                format!("-I{}", self.src_dir.join("event").display()),
            ]);
        }

        if self.http {
            bindings = bindings.clang_args(&[
                "-DNGX_HTTP".to_string(),
                format!("-I{}", self.src_dir.join("http").display()),
                format!("-I{}", self.src_dir.join("http/modules").display()),
                format!("-I{}", self.src_dir.join("http/v2").display()),
            ]);
        }

        if self.mail {
            bindings = bindings.clang_args(&[
                "-DNGX_MAIL".to_string(),
                format!("-I{}", self.src_dir.join("mail").display()),
            ]);
        }

        if self.stream {
            bindings = bindings.clang_args(&[
                "-DNGX_STREAM".to_string(),
                format!("-I{}", self.src_dir.join("stream").display()),
            ]);
        }

        let builder = bindings.generate()?;

        builder.write_to_file(self.out_file)?;

        Ok(())
    }
}
