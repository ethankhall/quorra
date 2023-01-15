use std::path::PathBuf;

use clap::{ColorChoice, Parser};

use crate::config::logging::*;

#[derive(Parser, Debug)]
#[clap(author, version, color = ColorChoice::Always)]
pub struct Opts {
    #[clap(flatten)]
    pub logging_opts: LoggingOpts,

    #[clap(flatten)]
    pub runtime_metrics: RuntimeArgs,

    #[clap(long, short)]
    pub config_file: PathBuf,
}
