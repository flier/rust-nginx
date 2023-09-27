use std::{
    env::current_dir,
    fs,
    process::{Command, Output},
};

use tracing::instrument;

use crate::{Error, Result};

pub trait CommandExt {
    fn run(&mut self) -> Result<Output>;
}

impl CommandExt for Command {
    #[instrument]
    fn run(&mut self) -> Result<Output> {
        let prog = self.get_program().to_string_lossy().to_string();
        let cur_dir = self
            .get_current_dir()
            .map_or_else(current_dir, |p| Ok(p.to_path_buf()))?;

        fs::write(cur_dir.join(format!("{}.sh", prog)), format!("{:?}", self))?;

        let out = self.output()?;

        fs::write(cur_dir.join(format!("{}.stdout", prog)), &out.stdout)?;
        fs::write(cur_dir.join(format!("{}.stderr", prog)), &out.stderr)?;

        if out.status.success() {
            Ok(out)
        } else {
            Err(Error::ExecuteError(out))
        }
    }
}
