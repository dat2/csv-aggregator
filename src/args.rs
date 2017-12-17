use clap::{Arg, App};
use failure::Error;
use glob::glob;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Arguments {
  pub config_file: PathBuf,
  pub input_files: Vec<PathBuf>,
}

pub fn parse_args() -> Result<Arguments, Error> {
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
        .takes_value(true),
    )
    .arg(Arg::with_name("input").required(true))
    .get_matches();

  let config_file = matches.value_of("config").unwrap_or("config.yaml").into();
  let input_files = glob(matches.value_of("input").unwrap())?
    .filter(|r| r.is_ok())
    .map(|r| r.unwrap())
    .collect();

  Ok(Arguments {
    config_file: config_file,
    input_files: input_files,
  })
}
