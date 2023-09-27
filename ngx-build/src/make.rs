use std::{path::PathBuf, process::Command};

use tracing::instrument;

use crate::{CommandExt, Result};

#[derive(Clone, Debug)]
pub struct Make {
    pub src_dir: PathBuf,
    pub build_dir: PathBuf,
}

impl Make {
    #[instrument]
    pub fn build(&self) -> Result<()> {
        Command::new("make")
            .current_dir(&self.src_dir)
            .run()
            .map(|_| ())
    }

    #[instrument]
    pub fn install(&self) -> Result<()> {
        Command::new("make")
            .arg("install")
            .current_dir(&self.src_dir)
            .run()
            .map(|_| ())
    }
}
