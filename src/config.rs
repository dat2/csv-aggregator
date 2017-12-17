use failure::Error;
use serde_yaml;
use std::fs::File;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum TypedField {
  Date { name: String, format: String },
  Number { name: String },
}

impl TypedField {
  fn name(&self) -> &str {
    match *self {
      TypedField::Date { ref name, .. } |
      TypedField::Number { ref name } => name,
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ConfigField {
  Typed(TypedField),
  Basic(String),
}

impl ConfigField {
  pub fn name(&self) -> &str {
    match *self {
      ConfigField::Typed(ref f) => f.name(),
      ConfigField::Basic(ref s) => s,
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
  pub fields: Vec<ConfigField>,
  pub sort: Option<String>,
  pub filter: Option<Vec<String>>
}

impl Config {
  pub fn get_sort_index(&self) -> Option<usize> {
    self.sort.clone().and_then(|sort_field_name| {
      self.fields.iter().rposition(|config_field| {
        config_field.name() == sort_field_name
      })
    })
  }
}

pub fn parse_config(path: &PathBuf) -> Result<Config, Error> {
  let file = File::open(path)?;
  let config = serde_yaml::from_reader(file)?;
  Ok(config)
}
