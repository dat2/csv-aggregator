use chrono::prelude::*;
use config::{Config, ConfigField, TypedField};
use csv::{QuoteStyle, ReaderBuilder, StringRecord, WriterBuilder};
use failure::Error;
use filter::Filter;
use float_cmp::{ApproxEqUlps, ApproxOrdUlps};
use serde::ser::{Serialize, Serializer};
use std::cmp::Ordering;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Record {
  fields: Vec<Field>,
  sort_params: Option<SortParams>,
}

impl PartialOrd for Record {
  fn partial_cmp(&self, other: &Record) -> Option<Ordering> {
    match (&self.sort_params, &other.sort_params) {
      // if both records are going to sort on the same fields
      // then we compare the fields specified
      (&Some(ref self_sort_params), &Some(ref other_sort_params))
        if self_sort_params.index == other_sort_params.index => {
        self.fields.get(self_sort_params.index).and_then(
          |self_field| {
            other.fields.get(other_sort_params.index).and_then(
              |other_field| {
                self_field.partial_cmp(other_field)
              },
            )
          },
        )
      }
      _ => None,
    }
  }
}

impl Ord for Record {
  fn cmp(&self, other: &Record) -> Ordering {
    self.partial_cmp(other).unwrap_or_else(|| Ordering::Equal)
  }
}

impl Record {
  fn new(config: &Config, string_record: &StringRecord) -> Result<Record, Error> {
    let sort_params = config.get_sort_index().map(SortParams::new);
    let mut fields = Vec::new();
    for (index, config_field) in config.fields.iter().enumerate() {
      fields.push(Field::new(config_field, &string_record[index])?);
    }
    Ok(Record {
      fields: fields,
      sort_params: sort_params,
    })
  }
}

#[derive(Clone, Debug)]
struct NaiveFloat {
  inner: f64
}

impl PartialEq for NaiveFloat {
  fn eq(&self, other: &NaiveFloat) -> bool {
    self.inner.approx_eq_ulps(&other.inner, 2)
  }
}
impl Eq for NaiveFloat {}

impl Ord for NaiveFloat {
  fn cmp(&self, other: &NaiveFloat) -> Ordering {
    self.inner.approx_cmp(&other.inner, 2)
  }
}
impl PartialOrd for NaiveFloat {
  fn partial_cmp(&self, other: &NaiveFloat) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl FromStr for NaiveFloat {
  type Err = ::std::num::ParseFloatError;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    f64::from_str(s).map(|f| NaiveFloat { inner: f })
  }
}

impl Serialize for NaiveFloat {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
  {
    self.inner.serialize(serializer)
  }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Serialize)]
enum Field {
  Date(NaiveDate),
  Number(NaiveFloat),
  String(String),
}

impl Field {
  fn new(config_field: &ConfigField, value: &str) -> Result<Field, Error> {
    match *config_field {
      ConfigField::Typed(ref f) => {
        match *f {
          TypedField::Date { ref format, .. } => Ok(Field::Date(
            NaiveDate::parse_from_str(value, format)?,
          )),
          TypedField::Number { .. } => {
            Ok(Field::Number(value.parse()?))
          }
        }
      }
      ConfigField::Basic(_) => Ok(Field::String(value.to_owned())),
    }
  }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct SortParams {
  index: usize,
}

impl SortParams {
  fn new(index: usize) -> SortParams {
    SortParams { index: index }
  }
}

pub fn parse_and_aggregate_csvs(paths: &[PathBuf], config: &Config, filter: &Option<Filter>) -> Result<Vec<Record>, Error> {
  let mut result = Vec::new();

  for path in paths {
    let mut reader = ReaderBuilder::new().has_headers(false).from_path(path)?;
    for string_record in reader.records() {
      let string_record = string_record?;
      if filter.clone().map(|f| f.matches(&string_record)).unwrap_or(true) {
        result.push(Record::new(config, &string_record)?);
      }
    }
  }

  Ok(result)
}

pub fn transform_aggregated_csv(_config: &Config, rows: &[Record]) -> Vec<Record> {
  let mut result = rows.to_vec();
  result.sort();
  result
}

pub fn write_aggregated_csv(rows: Vec<Record>) -> Result<(), Error> {
  let mut writer = WriterBuilder::new()
    .quote_style(QuoteStyle::NonNumeric)
    .from_writer(io::stdout());
  for row in rows {
    writer.serialize(row.fields)?;
  }
  Ok(())
}
