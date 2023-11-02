use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::bail;
use cargo_metadata::{Metadata, MetadataCommand};
use clap::{Parser, Subcommand};
use fs_extra::dir::{copy, CopyOptions};
use tracing::{debug, info, instrument, trace};

/// CI/CD workflows
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to `Cargo.toml`
    #[arg(long)]
    manifest_path: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Cmd,
}

impl Args {
    fn cargo(&self) -> Command {
        Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
    }

    fn cargo_metadata(&self) -> anyhow::Result<Metadata> {
        let mut cmd = MetadataCommand::new();

        if let Some(p) = self.manifest_path.as_ref() {
            cmd.manifest_path(p);
        }

        cmd.no_deps();

        Ok(cmd.exec()?)
    }

    fn env_vars(&self) -> BTreeMap<String, String> {
        env::vars().collect()
    }

    fn build_info(&self) -> &build_info::BuildInfo {
        build_info::build_info!(fn build);

        build()
    }
}

#[derive(Debug, Subcommand)]
enum Cmd {
    /// Generate binding file
    Gen,
    /// Build nginx and related crates
    Build,
    /// Test the runtime with static link
    Test,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let build_info = args.build_info();
    let metadata = args.cargo_metadata()?;
    let target_dir = metadata.target_directory.as_std_path();
    let profile_dir = target_dir.join(&build_info.profile);
    let build_dir = profile_dir.join("build");

    trace!(?args, env=?args.env_vars(), ?build_info, ?profile_dir);

    let mut cargo = args.cargo();

    match args.cmd {
        Cmd::Gen => {
            let cmd = cargo.args(["build", "--package", "ngx-sys", "--features", "gen"]);

            debug!(?cmd, "gen");

            let status = cmd.spawn()?.wait()?;

            if status.success() {
                let bindings_file = find_ngx_sys(&build_dir)?;
                let src_dir: PathBuf = metadata.workspace_root.as_std_path().join("ngx-sys/src");

                copy_binding_file(&bindings_file, &src_dir.join("bindings.rs"))?;
            } else {
                bail!("generate binding file failed");
            }
        }
        Cmd::Build => {
            let cmd = cargo.args(["build", "--package", "ngx-mod", "--examples"]);

            debug!(?cmd, "build");

            let status = cmd.spawn()?.wait()?;

            if status.success() {
                let nginx_dist_dir = find_ngx_src(&build_dir)?;
                let nginx_dir = profile_dir.join("nginx");
                copy_nginx(&nginx_dist_dir, &nginx_dir)?;

                let examples_dir = profile_dir.join("examples");
                let modules_dir = nginx_dir.join("modules");
                copy_nginx_modules(&examples_dir, &modules_dir)?;
            } else {
                debug!(?status);
            }
        }
        Cmd::Test => {
            let cmd = cargo.args(["test", "--features", "static-link"]);

            debug!(?cmd, "test");

            let status = cmd.spawn()?.wait()?;
        }
    }

    Ok(())
}

#[instrument(fields(result))]
fn find_ngx_sys(build_dir: &Path) -> anyhow::Result<PathBuf> {
    find_latest_crate(build_dir, "ngx-sys", |p| {
        if p.join("out/bindings.rs").is_file() {
            Some(p.join("out/bindings.rs"))
        } else {
            None
        }
    })
}

#[instrument(fields(result))]
fn find_ngx_src(build_dir: &Path) -> anyhow::Result<PathBuf> {
    find_latest_crate(build_dir, "ngx-src", |p| {
        if p.join("out/dist/sbin/nginx").is_file() {
            Some(p.join("out/dist"))
        } else {
            None
        }
    })
}

#[instrument(skip(build_dir, f), fields(result))]
fn find_latest_crate<F>(build_dir: &Path, name: &str, f: F) -> anyhow::Result<PathBuf>
where
    F: Fn(&Path) -> Option<PathBuf>,
{
    debug!(?build_dir, "finding crate `{}`...", name);

    let mut dirs = fs::read_dir(build_dir)?
        .flat_map(|e| e)
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.file_name().unwrap().to_string_lossy().starts_with(name))
        .flat_map(|p| f(&p))
        .collect::<Vec<_>>();

    dirs.sort_by_cached_key(|p| {
        p.metadata()
            .and_then(|md| md.modified())
            .expect("last modified time")
    });

    if let Some(p) = dirs.last() {
        info!(?p, "found `{}`", name);

        Ok(p.clone())
    } else {
        bail!("missing `{}`", name)
    }
}

fn copy_binding_file(from: &Path, to: &Path) -> io::Result<()> {
    info!(?from, ?to, "copy bindings file");

    fs::copy(from, to).map(|_| ())
}

#[instrument(skip(from, to))]
fn copy_nginx(from: &Path, to: &Path) -> anyhow::Result<()> {
    info!(?from, ?to);

    let mut opts = CopyOptions::new();

    opts.overwrite = true;
    opts.content_only = true;

    copy(from, to, &opts)?;

    Ok(())
}

#[instrument(skip(from, to))]
fn copy_nginx_modules(from: &Path, to: &Path) -> io::Result<()> {
    info!(?from, ?to);

    let modules = fs::read_dir(from)?
        .flat_map(|e| e)
        .map(|e| e.path())
        .flat_map(|file| {
            if file.is_file() && is_nginx_module(&file) {
                Some(file)
            } else {
                None
            }
        });

    fs::create_dir_all(to)?;

    for module in modules {
        let file_name = module.file_name().unwrap();
        let to = to.join(file_name);

        trace!(?file_name, "copy module");

        fs::copy(module, to)?;
    }

    Ok(())
}

fn is_nginx_module(file: &Path) -> bool {
    if let Some((name, ext)) = file.file_name().zip(file.extension()) {
        if !name.to_string_lossy().contains("-") && (ext == "so" || ext == "dylib" || ext == "dll")
        {
            return true;
        }
    }

    false
}
