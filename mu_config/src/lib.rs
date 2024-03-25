use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use toml::{Table, Value};

const ARCHITECTURES: [&str; 2] = ["common", "x64"];
const MODULES: [&str; 1] = ["common"];


/// A Serializavle/Deserializable toml file for platform
/// build configurations.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
  /// A lookup dictionary of library instances
  #[serde(alias = "libraryinstances", alias="LibraryInstances")]
  pub libraries: LibraryInstances,

}

/// A lookup dictionary of library instances based off the library name and architecture.
#[derive(Debug, Serialize, Default)]
pub struct LibraryInstances {
  pub instances: HashMap<(String, String), Instance>,
}

impl LibraryInstances {
  fn parse_table(&mut self, table: &Table, arch: &str) {
    for (name, value) in table.iter() {
      match value {
        Value::String(path) => {
          let key = (name.to_string(), arch.to_string());
          self.instances.insert(key, Instance {
            name: name.to_string(),
            path: path.to_string(),
            arch: arch.to_string(),
          });
        }
        Value::Table(table) if ARCHITECTURES.contains(&name.to_lowercase().as_str()) => {
          self.parse_table(table, name);
        }
        Value::Table(_table) if MODULES.contains(&name.to_lowercase().as_str()) => {
          println!("ERROR: filtering on module types not implemented yet.");
        }
        // TODO: Could have a table here in the future with other customizations
        // like Lib={path = "pkg1::...", family = "GCC5"}
        value => {println!("ERROR: unexpected value [{:?}]", value)}
      }
    }
  }
}

/// The actual library instance that is being used.
#[derive(Debug, Serialize)]
pub struct Instance {
  pub name: String,
  pub path: String,
  pub arch: String,
}

impl<'de> Deserialize<'de> for LibraryInstances {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
      D: serde::Deserializer<'de>,
  {
      let mut instances = LibraryInstances::default();
      let table: Table = Deserialize::deserialize(deserializer)?;
      instances.parse_table(&table, "common");
      Ok(instances)
  }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let data = include_str!("../tests/data/config.toml"); 
        let config = toml::from_str::<Config>(data).unwrap();
        assert!(config.libraries.instances.contains_key(&(String::from("AdvLib"), String::from("common"))));
        assert!(config.libraries.instances.contains_key(&(String::from("AdvLib"), String::from("x64"))));
    }
}