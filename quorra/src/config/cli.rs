use clap::{ColorChoice, Parser, Subcommand};

use crate::config::logging::*;

#[derive(Parser, Debug)]
#[clap(author, version, color = ColorChoice::Always)]
pub struct Opts {
    #[clap(flatten)]
    pub logging_opts: LoggingOpts,

    #[clap(flatten)]
    pub runtime_metrics: RuntimeArgs,

    #[clap(subcommand)]
    pub command: SubCommands,
}

#[derive(Subcommand, Debug)]
pub enum SubCommands {
    /// Run the primary web server mocking responses
    Server(crate::server::ServerCommandConfig),
    /// Convert a har file into usable config
    ConvertHar(crate::convert_har::ConvertHarCommandConfig),
}
