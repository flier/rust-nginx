use std::env::current_dir;
use std::fs;
use std::process::Command;

use tracing::instrument;

use crate::{Error, Result};

pub trait CommandExt {
    type Output;

    fn run(&mut self) -> Result<Self::Output>;
}

impl CommandExt for Command {
    type Output = ();

    #[instrument]
    fn run(&mut self) -> Result<()> {
        let prog = self.get_program().to_string_lossy().to_string();
        let cur_dir = self
            .get_current_dir()
            .map_or_else(current_dir, |p| Ok(p.to_path_buf()))?;

        fs::write(cur_dir.join(format!("{}.sh", prog)), format!("{:?}", self))?;

        self.stdout(fs::File::create(cur_dir.join(format!("{}.stdout", prog)))?)
            .stderr(fs::File::create(cur_dir.join(format!("{}.stderr", prog)))?);

        if self.spawn()?.wait()?.success() {
            Ok(())
        } else {
            Err(Error::ExecuteError)
        }
    }
}
