use std::path::PathBuf;

use crate::{Configure, Error, Result};

#[derive(Clone, Debug, Default)]
pub struct Builder {
    debug: bool,
    compat: bool,
    without_http: bool,
    without_http_cache: bool,
    mail: bool,
    stream: bool,
    threads: bool,
    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    file_aio: bool,
    modules: Vec<String>,
    without_modules: Vec<String>,
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
    pub fn with_debug(&mut self) -> &mut Self {
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

    /// disables multiple modules
    pub fn without_modules<I: IntoIterator<Item = S>, S: AsRef<str>>(
        &mut self,
        modules: I,
    ) -> &mut Self {
        self.without_modules
            .extend(modules.into_iter().map(|s| s.as_ref().to_string()));
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

    /// disable HTTP server
    pub fn without_http(&mut self) -> &mut Self {
        self.without_http = true;
        self
    }

    /// disable HTTP cache
    pub fn without_http_cache(&mut self) -> &mut Self {
        self.without_http_cache = true;
        self
    }

    /// enables building the mail module.
    pub fn with_mail(&mut self) -> &mut Self {
        self.mail = true;
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

    pub fn configure(self) -> Result<Configure> {
        Ok(Configure {
            debug: self.debug,
            compat: self.compat,
            without_http: self.without_http,
            without_http_cache: self.without_http_cache,
            mail: self.mail,
            stream: self.stream,
            threads: self.threads,
            #[cfg(any(target_os = "freebsd", target_os = "linux"))]
            file_aio: self.file_aio,
            modules: self.modules,
            without_modules: self.without_modules,
            src_dir: self
                .src_dir
                .ok_or_else(|| Error::MissingArgument("src_dir"))?,
            build_dir: self
                .build_dir
                .ok_or_else(|| Error::MissingArgument("build_dir"))?,
            out_dir: self
                .out_dir
                .ok_or_else(|| Error::MissingArgument("out_dir"))?,
            openssl_dir: self.openssl_dir,
            openssl_opt: self.openssl_opt,
            pcre_dir: self.pcre_dir,
            pcre_opt: self.pcre_opt,
            zlib_dir: self.zlib_dir,
            zlib_opt: self.zlib_opt,
        })
    }
}
