#![feature(conservative_impl_trait)]

extern crate chrono;
extern crate clap;
#[macro_use]
extern crate combine;
extern crate csv;
#[macro_use]
extern crate failure;
extern crate float_cmp;
extern crate glob;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

mod args;
mod config;
mod filter;
mod naive_float;
mod parser;
mod records;

use args::parse_args;
use config::parse_config;
use failure::Error;
use parser::parse_filter;
use records::{parse_and_aggregate_csvs, transform_aggregated_csv, write_aggregated_csv};

fn run() -> Result<(), Error> {
  let args = parse_args()?;
  let config = parse_config(&args.config_file)?;
  let filter = match config.filter {
    Some(ref filters) => Some(parse_filter(filters)?),
    None => None
  };
  let aggregated = parse_and_aggregate_csvs(&args.input_files, &config, &filter)?;
  let result = transform_aggregated_csv(&config, &aggregated);
  write_aggregated_csv(result)?;
  Ok(())
}

fn main() {
  if let Err(e) = run() {
    println!("{}", e);
  }
}
