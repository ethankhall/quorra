use clap::Parser;
use quorra_har::HarConvertor;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct ConvertHarCommandConfig {
    #[clap(long("har"))]
    /// Input HAR file
    pub har_file: PathBuf,

    #[clap(long("destination"))]
    /// Where the converted files should be created
    pub dest_directory: PathBuf,
}

impl ConvertHarCommandConfig {
    pub async fn run_convert(&self) -> Result<(), anyhow::Error> {
        std::fs::create_dir_all(&self.dest_directory)?;
        let convertor = HarConvertor::new(&self.har_file);
        convertor.convert(&self.dest_directory).await?;
        Ok(())
    }
}
