use super::{Component, Library};
use proc_macro2::TokenStream;
use syn::{Ident, Token};
use quote::{quote, ToTokens};

use mu_config::{Architecture, Config, LibraryInstance, Module};

use std::collections::HashMap;

mod kw {
    syn::custom_keyword!(path);
    syn::custom_keyword!(Path);
    syn::custom_keyword!(env);
    syn::custom_keyword!(Env);
}

pub fn parse(tokens: TokenStream) -> TokenStream {
    let mut parsed = match syn::parse2::<PathDescribed>(tokens) {
        Ok(component) => component,
        Err(e) => return e.to_compile_error().into(),
    };

    match parsed.resolve() {
        Ok(_) => (),
        Err(e) => return e.to_compile_error().into(),
    }

    parsed.to_token_stream()
}

struct PathDescribed {
    component: Component,
    impl_map: HashMap<String, Library>,
    config: Config,
}

impl PathDescribed {
    fn resolve(&mut self) -> syn::Result<()> {
        let component_name = self.component.name.to_string().to_lowercase();
        let component = self.config.components.get(&component_name).unwrap();

        let library_list: Vec<LibraryInstance> = self.component.library_list
            .iter()
            .map(|lib| lib.to_string().to_lowercase())
            .map(|lib| self.config.libraries.get(&lib, &component.arch, &component.module).unwrap())
            .collect();

        for library in library_list {
            let instance = syn::parse_str::<Library>(&format!("{}={}", &library.name, &library.path)).unwrap();
            self.register_dependencies(&instance, &component.arch, &component.module);
        }

        while !self.impl_map.values().all(|lib| lib.is_resolved()) {
            let temp_map = self.impl_map.clone();
        
            for lib in self.impl_map.values_mut() {
                lib.resolve(&temp_map).unwrap();
            }
      
            // If the map hasn't changed, we have a circular dependency
            if temp_map == self.impl_map {
                panic!("Circular dependency detected");
            }
        }

        Ok(())
    }

    fn register_dependencies(&mut self, library: &Library, arch: &Architecture, module: &Module) {
        let key = library.name.to_string().to_lowercase();
        if self.impl_map.contains_key(&key) {
            return;
        }
        self.impl_map.insert(key, library.clone());

        for library in &library.required {
            let library = self.config.libraries.get(&library.to_string().to_lowercase(), arch, module).unwrap();
            let instance = syn::parse_str::<Library>(&format!("{}={}", &library.name, &library.path)).unwrap();
            self.register_dependencies(&instance, arch, module);
        }
    }
}

impl syn::parse::Parse for PathDescribed {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let component = input.parse::<Component>()?;
        
        let mut path: Option<String> = None;
        if input.peek(kw::path) || input.peek(kw::Path) {
            input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;
            path = Some(input.parse::<syn::LitStr>()?.value());
            input.parse::<Token![;]>()?;
        }
        else if input.peek(kw::env) || input.peek(kw::Env) {
            input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;
            let env = input.parse::<syn::LitStr>()?;
            input.parse::<Token![;]>()?;
            path = Some(std::env::var(env.value()).unwrap());
        }

        if path.is_none() {
            return Err(input.error("Expected 'Path' or 'Env' keyword"));
        }

        let toml_content = std::fs::read_to_string(path.unwrap()).map_err(
            |e| syn::Error::new(input.span(), e)
        )?;
        let config = toml::from_str::<Config>(&toml_content).unwrap();

        Ok(PathDescribed {
            component,
            impl_map: HashMap::new(),
            config,
        })
    }
}

impl ToTokens for PathDescribed {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.component.name;

        let mut library_list = vec![];
        for library in &self.component.library_list {
          let lib = self.impl_map.get(&library.to_string().to_lowercase()).unwrap();
          library_list.push(lib);
        }
    
        tokens.extend(quote! {
          #name<#(#library_list),*>
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_parse1() {
        let expected = quote! {
            MyDriver<DebugLibBase>
        };
      
        let input = quote! {
            MyDriver<DebugLib>;
            Path = "tests/data/test_config1.toml";
        };
      
        let actual = parse(input);
        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn test_full_parse2() {
        let expected_output = quote! {
            MyDriver < DebugLibBase < PrintLibBase > >
        };
      
        let input = quote! {
            MyDriver<DebugLib>;
            Path = "tests/data/test_config2.toml";
        };

        let actual = parse(input);
        assert_eq!(actual.to_string(), expected_output.to_string());
    }

    #[test]
    fn test_full_parse3() {
        let expected_output = quote! {
            MyDriver < DebugLibBase < PrintLibBase >, AdvLibBase < PrintLibBase> >
        };

        let input = quote! {
            MyDriver<DebugLib, AdvLib>;
            Path = "tests/data/test_config2.toml";
        };

        let actual = parse(input);
        assert_eq!(actual.to_string(), expected_output.to_string());
    }

    #[test]
    fn test_full_parse4() {
        let expected_output = quote! {
            MyDriver < DebugLibBase < PrintLibBase >, AdvLibBase < PrintLibBase, WriteLibBase > >
        };

        let input = quote! {
            MyDriver<DebugLib, AdvLib>;
            Path = "tests/data/test_config3.toml";
        };
        
        let actual = parse(input);
        assert_eq!(actual.to_string(), expected_output.to_string());
    }

    #[test]
    fn test_full_parse5() {
        let expected_output = quote! {
            MyDriver < pk1::library::DebugLibBase, pk1::library::AdvLibSpecial < pk2::library::WriteLibBase, pk1::library::DebugLibBase > >
        };

        let input = quote! {
            MyDriver<DebugLib, AdvLib>;
            Path = "tests/data/test_config4.toml";
        };

        let actual = parse(input);
        assert_eq!(actual.to_string(), expected_output.to_string());
    }
}
