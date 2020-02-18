use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use anyhow::Result;
use tokio::signal;
#[macro_use]
extern crate slog;
use clap::Clap;
use schemars::schema_for;
use serde_json;

use singular_lib as lib;
use lib::util;

#[derive(Clap)]
#[clap(
    name = "singular",
    version = "0.1.0",
    author = "Gero Posmyk-Leinemann <gero.posmyk-leinemann@typefox.io>",
    about = "A singularizing reverse-proxy for kubernetes"
)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(
        name = "run",
        about = "Runs the actual proxy"
    )]
    Run(Run),
    #[clap(
        name = "generate-schema",
        about = "Generates the schema for the config file"
    )]
    GenerateSchema(GenerateSchema),
}

#[derive(Clap)]
struct Run {
    #[clap(
        short = "c",
        long = "config",
        default_value = "./config.json",
        help = "The path to the config file to use"
    )]
    config: String,
}

#[derive(Clap)]
struct GenerateSchema {
    #[clap(
        short = "o",
        long = "outFile",
        help = "The path where to write the schema to (default: stdout)"
    )]
    out_file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Run(r) => run(r).await,
        SubCommand::GenerateSchema(s) => generate_schema(s),
    }
}

async fn run(opts: Run) -> Result<(), Box<dyn std::error::Error>> {
    let log = util::create_logger();

    debug!(log, "using config: {}", opts.config);
    let config_path = PathBuf::from(opts.config);
    let config = lib::Config::load_config(&config_path)?;

    lib::do_run_singular(config, log.clone()).await?;

    signal::ctrl_c().await?;
    info!(log, "sigint received, quitting.");

    Ok(())
}

fn generate_schema(opts: GenerateSchema) -> Result<(), Box<dyn std::error::Error>> {
    let schema = schema_for!(lib::Config);
    let content = serde_json::to_string_pretty(&schema)?;

    match opts.out_file {
        None => println!("{}", content),
        Some(o) => {
            let out_path = PathBuf::from(o);
            let mut file = File::create(out_path)?;
            file.write_all(content.as_bytes())?;
        }
    }

    Ok(())
}
