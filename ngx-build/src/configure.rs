use std::{fs, path::PathBuf, process::Command};

use pathdiff::diff_paths;
use tracing::{debug, instrument};

use crate::{CommandExt, Make, Result};

#[derive(Clone, Debug)]
pub struct Configure {
    pub debug: bool,
    pub compat: bool,
    pub without_http: bool,
    pub without_http_cache: bool,
    pub mail: bool,
    pub stream: bool,
    pub threads: bool,
    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    pub file_aio: bool,
    pub modules: Vec<String>,
    pub without_modules: Vec<String>,
    pub src_dir: PathBuf,
    pub build_dir: PathBuf,
    pub out_dir: PathBuf,
    pub openssl_dir: Option<PathBuf>,
    pub openssl_opt: Option<String>,
    pub pcre_dir: Option<PathBuf>,
    pub pcre_opt: Option<String>,
    pub zlib_dir: Option<PathBuf>,
    pub zlib_opt: Option<String>,
}

impl Configure {
    #[instrument]
    pub fn run(self) -> Result<Make> {
        let mut args = vec![
            format!(
                "--builddir={}",
                diff_paths(&self.build_dir, &self.src_dir)
                    .expect("builddir")
                    .display()
            ),
            format!(
                "--prefix={}",
                diff_paths(&self.out_dir, &self.src_dir)
                    .expect("prefix")
                    .display()
            ),
        ];

        if self.debug {
            args.push("--with-debug".to_string());
        }
        if self.compat {
            args.push("--with-compat".to_string());
        }
        if self.without_http {
            args.push("--without-http".to_string());
        }
        if self.without_http_cache {
            args.push("--without-http-cache".to_string());
        }
        if self.mail {
            args.push("--with-mail".to_string());
        }
        if self.stream {
            args.push("--with-stream".to_string());
        }
        if self.threads {
            args.push("--with-threads".to_string());
        }
        #[cfg(any(target_os = "freebsd", target_os = "linux"))]
        if self.file_aio {
            args.push("--with-file-aio".to_string());
        }

        if let Some(dir) = self.openssl_dir {
            args.push(format!("--with-openssl={}", dir.display()));
            if let Some(opt) = self.openssl_opt {
                args.push(format!("--with-openssl-opt={}", opt));
            }
        }
        if let Some(dir) = self.pcre_dir {
            args.push(format!("--with-pcre={}", dir.display()));
            if let Some(opt) = self.pcre_opt {
                args.push(format!("--with-pcre-opt={}", opt));
            }
        }
        if let Some(dir) = self.zlib_dir {
            args.push(format!("--with-zlib={}", dir.display()));
            if let Some(opt) = self.zlib_opt {
                args.push(format!("--with-zlib-opt={}", opt));
            }
        }

        args.extend(
            self.modules
                .into_iter()
                .map(|m| format!("--with-{}_module", m)),
        );
        args.extend(
            self.without_modules
                .into_iter()
                .map(|m| format!("--without-{}_module", m)),
        );

        fs::create_dir_all(&self.build_dir)?;
        fs::create_dir_all(&self.out_dir)?;

        let mut cmd = Command::new(self.src_dir.join("configure"));

        cmd.args(&args).current_dir(&self.src_dir);

        debug!(?cmd);

        cmd.run().map(|_| Make {
            src_dir: self.src_dir,
            build_dir: self.build_dir,
        })
    }
}
