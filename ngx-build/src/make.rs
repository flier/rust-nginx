use std::{path::PathBuf, process::Command};

use tracing::{debug, instrument};

use crate::{CommandExt, Result};

#[derive(Clone, Debug)]
pub struct Make {
    pub src_dir: PathBuf,
    pub build_dir: PathBuf,
}

impl Make {
    #[instrument]
    pub fn build(&self) -> Result<()> {
        let mut cmd = Command::new("make");

        cmd.current_dir(&self.src_dir);

        debug!(?cmd, "build");

        cmd.run().map(|_| ())
    }

    #[instrument]
    pub fn install(&self) -> Result<()> {
        let mut cmd = Command::new("make");

        cmd.arg("install").current_dir(&self.src_dir);

        debug!(?cmd, "install");

        cmd.run().map(|_| ())
    }
}
