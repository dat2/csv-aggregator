extern crate clap;
extern crate failure;
extern crate glob;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use clap::{Arg, App};
use failure::Error;
use glob::glob;
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug)]
enum OutputMode {
  File(PathBuf),
  Stdout
}

#[derive(Debug)]
struct Arguments {
  output_mode: OutputMode,
  config_file: PathBuf,
  input_files: Vec<PathBuf>
}

fn parse_args() -> Result<Arguments, Error> {
  let matches = App::new("csv-aggregator")
    .version("0.1")
    .author("Nick D. <nickdujay@gmail.com>")
    .about("Aggregates cvs into a single csv file")
    .arg(
      Arg::with_name("config")
        .short("c")
        .long("config")
        .value_name("CONFIG_FILE")
        .help("Sets the config file")
        .takes_value(true)
    )
    .arg(
      Arg::with_name("output")
        .short("o")
        .long("output_file")
        .value_name("OUTPUT_FILE")
        .help("Sets the output file")
        .takes_value(true)
    )
    .arg(
      Arg::with_name("input")
        .required(true)
    )
    .get_matches();

  let output_mode = match matches.value_of("output") {
    Some(file) => OutputMode::File(file.into()),
    None => OutputMode::Stdout
  };
  let config_file = matches.value_of("config").unwrap_or("config.yaml").into();
  let input_files = glob(matches.value_of("input").unwrap())?
    .filter(|r| r.is_ok())
    .map(|r| r.unwrap())
    .collect();

  Ok(Arguments {
    output_mode: output_mode,
    config_file: config_file,
    input_files: input_files
  })
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Field {
  Date {
    name: String,
    format: String
  },
  Number {
    name: String
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum FieldKind {
  Field(Field),
  Name(String)
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
  fields: Vec<FieldKind>,
  sort: Option<String>,
}

fn parse_config(path: &PathBuf) -> Result<Config, Error> {
  let file = File::open(path)?;
  let config = serde_yaml::from_reader(file)?;
  Ok(config)
}

fn run() -> Result<(), Error> {
  let args = parse_args()?;

  let config = parse_config(&args.config_file);
  println!("{:?}", config);
  Ok(())
}

fn main() {
  if let Err(e) = run() {
    println!("{:?}", e);
  }
}
