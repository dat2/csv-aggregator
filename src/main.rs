#![feature(conservative_impl_trait)]

extern crate chrono;
extern crate clap;
extern crate csv;
extern crate failure;
extern crate glob;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use chrono::prelude::*;
use clap::{Arg, App};
use csv::{QuoteStyle, ReaderBuilder, StringRecord, WriterBuilder};
use failure::Error;
use glob::glob;
use rayon::prelude::*;
use std::cmp::{Ordering, PartialOrd};
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug)]
enum OutputMode {
  File(PathBuf),
  Stdout,
}

#[derive(Debug)]
struct Arguments {
  output_mode: OutputMode,
  config_file: PathBuf,
  input_files: Vec<PathBuf>,
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
        .takes_value(true),
    )
    .arg(
      Arg::with_name("output")
        .short("o")
        .long("output_file")
        .value_name("OUTPUT_FILE")
        .help("Sets the output file")
        .takes_value(true),
    )
    .arg(Arg::with_name("input").required(true))
    .get_matches();

  let output_mode = match matches.value_of("output") {
    Some(file) => OutputMode::File(file.into()),
    None => OutputMode::Stdout,
  };
  let config_file = matches.value_of("config").unwrap_or("config.yaml").into();
  let input_files = glob(matches.value_of("input").unwrap())?
    .filter(|r| r.is_ok())
    .map(|r| r.unwrap())
    .collect();

  Ok(Arguments {
    output_mode: output_mode,
    config_file: config_file,
    input_files: input_files,
  })
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Field {
  Date { name: String, format: String },
  Number { name: String },
}

impl Field {
  fn name<'a>(&'a self) -> &'a str {
    match self {
      &Field::Date { ref name, .. } => name,
      &Field::Number { ref name } => name,
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum FieldKind {
  Field(Field),
  Name(String),
}

impl FieldKind {
  fn name<'a>(&'a self) -> &'a str {
    match self {
      &FieldKind::Field(ref f) => f.name(),
      &FieldKind::Name(ref s) => s,
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
  fields: Vec<FieldKind>,
  sort: Option<String>,
}

impl Config {
  fn get_sort_field<'a>(&'a self) -> Option<(usize, &'a FieldKind)> {
    if let Some(ref sort_field_name) = self.sort {
      self
        .fields
        .iter()
        .rposition(|ref field_kind| field_kind.name() == sort_field_name)
        .map(|index| (index, &self.fields[index]))
    } else {
      None
    }
  }
}

fn parse_config(path: &PathBuf) -> Result<Config, Error> {
  let file = File::open(path)?;
  let config = serde_yaml::from_reader(file)?;
  Ok(config)
}

fn parse_csv_files(paths: &Vec<PathBuf>) -> Vec<StringRecord> {
  paths
    .par_iter()
    .map(|path| {
      ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .unwrap()
        .into_records()
        .collect()
    })
    .flat_map(|vec: Vec<_>| vec.into_par_iter().filter_map(|r| r.ok()))
    .collect()
}

fn transform(config: Config, rows: Vec<StringRecord>) -> Vec<Vec<String>> {
  // transform into a vec of strings
  let mut rows: Vec<Vec<_>> = rows.iter()
    .map(|record| record.into_iter().map(|s| s.trim().to_owned()).collect())
    .collect();

  // sort
  if let Some((index, field_kind)) = config.get_sort_field() {
    rows.par_sort_by(|a, b| {
      let a_field = &a[index];
      let b_field = &b[index];

      match field_kind {
        &FieldKind::Field(ref field) => {
          match field {
            &Field::Date { ref format, .. } => {
              let a_date = NaiveDate::parse_from_str(a_field, format).unwrap();
              let b_date = NaiveDate::parse_from_str(b_field, format).unwrap();
              a_date.cmp(&b_date)
            }
            &Field::Number { .. } => {
              let a_num: f64 = a_field.parse().unwrap();
              let b_num: f64 = b_field.parse().unwrap();
              a_num.partial_cmp(&b_num).unwrap_or(Ordering::Equal)
            }
          }
        }
        &FieldKind::Name(_) => a_field.cmp(b_field),
      }
    });
  }
  rows
}

fn output(output_mode: OutputMode, rows: Vec<Vec<String>>) -> Result<(), Error> {
  let writer: Box<Write> = match output_mode {
    OutputMode::File(path) => Box::new(File::open(path)?),
    OutputMode::Stdout => Box::new(io::stdout()),
  };
  let mut writer = WriterBuilder::new().quote_style(QuoteStyle::NonNumeric).from_writer(writer);
  for row in rows {
    writer.write_record(row.into_iter())?;
  }
  Ok(())
}

fn run() -> Result<(), Error> {
  let args = parse_args()?;
  let config = parse_config(&args.config_file)?;
  let aggregated = parse_csv_files(&args.input_files);
  let result = transform(config, aggregated);
  output(args.output_mode, result)?;
  Ok(())
}

fn main() {
  if let Err(e) = run() {
    println!("{:?}", e);
  }
}
