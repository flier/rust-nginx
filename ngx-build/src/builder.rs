use std::fs::{self, create_dir_all};
use std::path::{Path, PathBuf};
use std::process::Command;

use tracing::{debug, instrument};

use crate::{Error, Result};

#[derive(Clone, Debug, Default)]
pub struct Builder {
    debug: bool,
    compat: bool,
    stream: bool,
    threads: bool,
    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    file_aio: bool,
    modules: Vec<String>,
    src_dir: Option<PathBuf>,
    build_dir: Option<PathBuf>,
    out_dir: Option<PathBuf>,
    openssl_dir: Option<PathBuf>,
    openssl_opt: Option<String>,
    pcre_dir: Option<PathBuf>,
    pcre_opt: Option<String>,
    zlib_dir: Option<PathBuf>,
    zlib_opt: Option<String>,
}

impl Builder {
    /// enables the debugging log.
    pub fn debug(&mut self) -> &mut Self {
        self.debug = true;
        self
    }

    /// enables a module
    pub fn with_module<S: Into<String>>(&mut self, module: S) -> &mut Self {
        self.modules.push(module.into());
        self
    }

    /// enables multiple modules
    pub fn with_modules<I: IntoIterator<Item = S>, S: AsRef<str>>(
        &mut self,
        modules: I,
    ) -> &mut Self {
        self.modules
            .extend(modules.into_iter().map(|s| s.as_ref().to_string()));
        self
    }

    /// disables a module
    pub fn without_module<S: AsRef<str>>(&mut self, name: S) -> &mut Self {
        self.modules.retain(|m| m != name.as_ref());
        self
    }

    /// sets the path to the nginx sources.
    pub fn src_dir<P: Into<PathBuf>>(&mut self, dir: P) -> &mut Self {
        self.src_dir = Some(dir.into());
        self
    }

    /// sets the path to the OpenSSL library sources.
    pub fn openssl_dir<P: Into<PathBuf>>(&mut self, dir: P) -> &mut Self {
        self.openssl_dir = Some(dir.into());
        self
    }

    /// sets additional build options for OpenSSL.
    pub fn openssl_opt<S: Into<String>>(&mut self, opt: S) -> &mut Self {
        self.openssl_opt = Some(opt.into());
        self
    }

    /// sets the path to the sources of the PCRE library.
    pub fn pcre_dir<P: Into<PathBuf>>(&mut self, dir: P) -> &mut Self {
        self.pcre_dir = Some(dir.into());
        self
    }

    /// sets additional build options for PCRE.
    pub fn pcre_opt<S: Into<String>>(&mut self, opt: S) -> &mut Self {
        self.pcre_opt = Some(opt.into());
        self
    }

    /// sets the path to the sources of the zlib library.
    pub fn zlib_dir<P: Into<PathBuf>>(&mut self, dir: P) -> &mut Self {
        self.zlib_dir = Some(dir.into());
        self
    }

    /// sets additional build options for zlib.
    pub fn zlib_opt<S: Into<String>>(&mut self, opt: S) -> &mut Self {
        self.zlib_opt = Some(opt.into());
        self
    }

    /// sets a build directory.
    pub fn build_dir<P: Into<PathBuf>>(&mut self, dir: P) -> &mut Self {
        self.build_dir = Some(dir.into());
        self
    }

    /// defines a directory that will keep server files.
    pub fn out_dir<P: Into<PathBuf>>(&mut self, dir: P) -> &mut Self {
        self.out_dir = Some(dir.into());
        self
    }

    /// enables dynamic modules compatibility.
    pub fn with_compat(&mut self) -> &mut Self {
        self.compat = true;
        self
    }

    /// enables building the stream module for generic TCP/UDP proxying and load balancing.
    pub fn with_stream(&mut self) -> &mut Self {
        self.stream = true;
        self
    }

    /// enables the use of thread pools.
    pub fn with_threads(&mut self) -> &mut Self {
        self.threads = true;
        self
    }

    /// enables the use of asynchronous file I/O (AIO) on FreeBSD and Linux.
    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    pub fn with_file_aio(&mut self) -> &mut Self {
        self.file_aio = true;
        self
    }

    fn configure(&self) -> Result<()> {
        let cmd = Configure {
            debug: self.debug,
            compat: self.compat,
            stream: self.stream,
            threads: self.threads,
            #[cfg(any(target_os = "freebsd", target_os = "linux"))]
            file_aio: self.file_aio,
            modules: self.modules.as_slice(),
            src_dir: self
                .src_dir
                .as_ref()
                .map(|p| p.as_path())
                .ok_or_else(|| Error::MissingArgument("src_dir"))?,
            build_dir: self
                .build_dir
                .as_ref()
                .map(|p| p.as_path())
                .ok_or_else(|| Error::MissingArgument("build_dir"))?,
            out_dir: self
                .out_dir
                .as_ref()
                .map(|p| p.as_path())
                .ok_or_else(|| Error::MissingArgument("out_dir"))?,
            openssl_dir: self.openssl_dir.as_ref().map(|p| p.as_path()),
            openssl_opt: self.openssl_opt.as_ref().map(|s| s.as_str()),
            pcre_dir: self.pcre_dir.as_ref().map(|p| p.as_path()),
            pcre_opt: self.pcre_opt.as_ref().map(|s| s.as_str()),
            zlib_dir: self.zlib_dir.as_ref().map(|p| p.as_path()),
            zlib_opt: self.zlib_opt.as_ref().map(|s| s.as_str()),
        };

        cmd.run()
    }

    fn make(&self) -> Result<()> {
        let cmd = Make {
            src_dir: self
                .src_dir
                .as_ref()
                .map(|p| p.as_path())
                .ok_or_else(|| Error::MissingArgument("src_dir"))?,
            build_dir: self
                .build_dir
                .as_ref()
                .map(|p| p.as_path())
                .ok_or_else(|| Error::MissingArgument("build_dir"))?,
        };

        cmd.run()
    }

    /// build the nginx sources
    pub fn build(self) -> Result<()> {
        self.configure()?;
        self.make()?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Configure<'a> {
    debug: bool,
    compat: bool,
    stream: bool,
    threads: bool,
    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    file_aio: bool,
    modules: &'a [String],
    src_dir: &'a Path,
    build_dir: &'a Path,
    out_dir: &'a Path,
    openssl_dir: Option<&'a Path>,
    openssl_opt: Option<&'a str>,
    pcre_dir: Option<&'a Path>,
    pcre_opt: Option<&'a str>,
    zlib_dir: Option<&'a Path>,
    zlib_opt: Option<&'a str>,
}

impl<'a> Configure<'a> {
    #[instrument]
    pub fn run(self) -> Result<()> {
        let mut args = vec![
            format!("--builddir={}", self.build_dir.display()),
            format!("--prefix={}", self.out_dir.display()),
        ];

        if self.debug {
            args.push("--with-debug".to_string());
        }
        if self.compat {
            args.push("--with-compat".to_string());
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

        let mut cmd = Command::new(self.src_dir.join("configure").as_path());

        cmd.current_dir(&self.src_dir).args(&args);

        create_dir_all(&self.build_dir)?;
        create_dir_all(&self.out_dir)?;

        fs::write(&self.build_dir.join("configure.sh"), format!("{:?}", cmd))?;

        debug!(?cmd);

        let out = cmd.output()?;

        fs::write(&self.build_dir.join("configure.stdout"), &out.stdout)?;
        fs::write(&self.build_dir.join("configure.stderr"), &out.stderr)?;

        if out.status.success() {
            Ok(())
        } else {
            Err(Error::ConfigureError {
                stdout: String::from_utf8(out.stdout)?,
                stderr: String::from_utf8(out.stderr)?,
            })
        }
    }
}

#[derive(Clone, Debug)]
struct Make<'a> {
    src_dir: &'a Path,
    build_dir: &'a Path,
}

impl<'a> Make<'a> {
    #[instrument]
    pub fn run(self) -> Result<()> {
        let mut cmd = Command::new("make");

        debug!(?cmd);

        let out = cmd.current_dir(&self.src_dir).output()?;

        fs::write(&self.build_dir.join("make.stdout"), &out.stdout)?;
        fs::write(&self.build_dir.join("make.stderr"), &out.stderr)?;

        if out.status.success() {
            Ok(())
        } else {
            Err(Error::ConfigureError {
                stdout: String::from_utf8(out.stdout)?,
                stderr: String::from_utf8(out.stderr)?,
            })
        }
    }
}
