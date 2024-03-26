use std::collections::HashMap;
use mu_config::Architecture;
use syn::{Ident, Token};

use super::Library;

mod kw {
    syn::custom_keyword!(path);
    syn::custom_keyword!(Path);
    syn::custom_keyword!(env);
    syn::custom_keyword!(Env);
}

/// Parses the expected `HashMap<String, Library.` data using data provided by
/// the macro tokens.
pub struct FromMacro(HashMap<String, Library>);
impl syn::parse::Parse for FromMacro {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let impl_map = input.parse_terminated(Library::parse, Token![;])?;
      let impl_map: HashMap<String, Library> = impl_map
        .into_iter()
        .map(|lib| (lib.name.to_string(), lib))
        .collect();
    Ok(FromMacro(impl_map))
  }
}
impl Into<HashMap<String, Library>> for FromMacro {
  fn into(self) -> HashMap<String, Library> {
    self.0
  }
}

/// Parses the expected [HashMap<String, Library] from a configuration file
/// which is located at the destination described by the [kw::path] or
/// [kw::Path] keywords in the macro.
pub struct FromPath(HashMap<String, Library>);
impl syn::parse::Parse for FromPath {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    if !(input.peek(kw::Path) || input.peek(kw::path)) {
      return Err(input.error("Expected 'path' keyword"));
    }

    input.parse::<Ident>()?;
    input.parse::<Token![=]>()?;
    let path = input.parse::<syn::LitStr>()?;
    input.parse::<Token![;]>()?;

    // TODO: Parse with actual Config parser eventually
    let mut map = HashMap::new();
    let toml_content = std::fs::read_to_string(path.value())
    .map_err(
        |e| syn::Error::new(path.span(), e)
    )?;
    let config: mu_config::Config = toml::from_str(&toml_content).unwrap();
    
    for (key, instance) in config.libraries.instances {
      // TODO, Find the specified architecture for the component
      if key.arch == Architecture::Common {
        let lib = syn::parse_str::<Library>(&format!("{}={}", key.name, instance.path)).unwrap();
        map.insert(key.name, lib);
      }
    }
    Ok(FromPath(map))
  }
}

impl Into<HashMap<String, Library>> for FromPath {
  fn into(self) -> HashMap<String, Library> {
    self.0
  }
}

pub struct FromEnv(HashMap<String, Library>);
impl syn::parse::Parse for FromEnv {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    if !(input.peek(kw::env) || input.peek(kw::Env)) {
      return Err(input.error("Expected 'env' keyword"));
    }

    input.parse::<Ident>()?;
    input.parse::<Token![=]>()?;
    let env = input.parse::<syn::LitStr>()?;
    input.parse::<Token![;]>()?;

    let path = std::env::var(env.value()).unwrap();

    // TODO: Parse with actual Config Parser eventually
    let mut map = HashMap::new();
    let toml_content = std::fs::read_to_string(path)
    .map_err(
        |e| syn::Error::new(env.span(), e)
    )?;
    let config: mu_config::Config = toml::from_str(&toml_content).unwrap();
    for (key, instance) in config.libraries.instances {
      // TODO, Find the specified architecture for the component
      if key.arch == Architecture::Common {
        let lib = syn::parse_str::<Library>(&format!("{}={}", key.name, instance.path)).unwrap();
        map.insert(key.name, lib);
      }
    }
    Ok(FromEnv(map))
    }
}


impl Into<HashMap<String, Library>> for FromEnv {
    fn into(self) -> HashMap<String, Library> {
      self.0
    }
}
