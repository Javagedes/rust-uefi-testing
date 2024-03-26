use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use toml::{Table, Value};

pub mod types;
pub use types::{Architecture, Module};



/// A Serializavle/Deserializable toml file for platform
/// build configurations.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
  /// A lookup dictionary of library instances
  #[serde(alias = "libraryinstances", alias="LibraryInstances")]
  pub libraries: LibraryInstances,

}

/// The actual library instance that is being used.
#[derive(Debug, Serialize, Clone)]
pub struct Instance {
    pub name: String,
    pub path: String,
}

/// A lookup dictionary of library instances based off the library name and architecture.
#[derive(Debug, Serialize, Default)]
pub struct LibraryInstances {
  instances: HashMap<LibraryKey, Instance>,
}

impl LibraryInstances {
    fn merge(&mut self, other: LibraryInstances) {
        self.instances.extend(other.instances);
    }

    pub fn get(&self, name: &str, arch: Architecture, module: Module) -> Option<Instance> {
        let search_order = [
            LibraryKey { name: name.to_lowercase(), arch: arch.clone(), module: module.clone() },
            LibraryKey { name: name.to_lowercase(), arch: arch.clone(), module: Module::Common },
            LibraryKey { name: name.to_lowercase(), arch: Architecture::Common, module: module.clone() },
            LibraryKey { name: name.to_lowercase(), arch: Architecture::Common, module: Module::Common },
        ];

        for instance in search_order.iter() {
            if let Some(instance) = self.instances.get(instance) {
                return Some(instance.clone())
            }
        }
        None
    }
}

impl<'de> Deserialize<'de> for LibraryInstances {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut instances = LibraryInstances::default();

        if let Some(arr) = Value::deserialize(deserializer)?.as_array() {
            for table in arr {
                let table = table.as_table().unwrap();
                instances.merge(process_library_table(table).unwrap());
            }
        }
        println!("{:?}", instances.instances);
        Ok(instances)
    }
}

fn process_library_table(table: &Table) -> Result<LibraryInstances, ()>
{
    let mut library_list: Vec<(&str, &str)> = vec![];
    let mut arch_list: Vec<Architecture> = vec![];
    let mut module_list: Vec<Module> = vec![];

    // First loop, find the architecture and module values
    for (name, value) in table.iter() {

        if name.to_lowercase() == "arch" {
            arch_list = value.as_array()
                .unwrap()
                .iter()
                .map(|arch| arch.try_into().unwrap())
                .collect();
        }
        else if name.to_lowercase() == "module" {
            module_list = value.as_array()
                .unwrap()
                .iter()
                .map(|module| module.try_into().unwrap())
                .collect();
        }
        else {
            library_list.push((name, value.as_str().unwrap()));
        }

    }

    // If no arch or module values are found, default to common
    if arch_list.len() == 0 {
        arch_list.push(Architecture::Common);
    }

    if module_list.len() == 0 {
        module_list.push(Module::Common);
    }

    let mut instances: HashMap<LibraryKey, Instance> = HashMap::new();
    for arch in &arch_list {
        for module in &module_list {
            for (name, path) in library_list.iter() {
                let key = LibraryKey {
                    name: name.to_lowercase(),
                    arch: arch.clone(),
                    module: module.clone(),
                };
                instances.insert(key, Instance {
                    name: name.to_lowercase(),
                    path: path.to_string(),
                });
            }
        }
    }

    Ok(LibraryInstances { instances})
}

#[derive(Debug, PartialEq, Eq, Hash, Serialize)]
struct LibraryKey {
    name: String,
    arch: Architecture,
    module: Module,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let data = include_str!("../tests/data/config.toml"); 
        let config = toml::from_str::<Config>(data).unwrap();

        assert_eq!(
            config.libraries.get("AdvLib", Architecture::Common, Module::Common).unwrap().path,
            "pkg1::library::AdvLibBase"
        );

        assert_eq!(
            config.libraries.get("AdvLib", Architecture::X64, Module::Common).unwrap().path,
            "pkg1::library::AdvLibX64"
        );

        assert_eq!(
            config.libraries.get("AdvLib", Architecture::X64, Module::DxeDriver).unwrap().path,
            "pkg1::library::AdvLibDxeOpt"
        );
    }
}