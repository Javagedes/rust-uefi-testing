extern crate proc_macro;

use std::collections::HashMap;


use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, token::Comma, Ident, Token};

mod map;

type FullyDescribed = TokenParser<map::FromMacro>;
type PathDescribed = TokenParser<map::FromPath>;
type EnvDescribed = TokenParser<map::FromEnv>;

#[proc_macro]
pub fn component(tokens: TokenStream) -> TokenStream {
    let mut parsed = match syn::parse::<FullyDescribed>(tokens) {
        Ok(component) => component,
        Err(e) => return e.to_compile_error().into(),
    };

    match parsed.resolve() {
        Ok(_) => (),
        Err(e) => return e.to_compile_error().into(),
    }

    let tokens: proc_macro2::TokenStream = parsed.to_token_stream();
    tokens.into()
}

#[proc_macro]
pub fn component_from_path(tokens: TokenStream) -> TokenStream {
  let mut parsed = match syn::parse::<PathDescribed>(tokens) {
    Ok(component) => component,
    Err(e) => return e.to_compile_error().into(),
  };

  match parsed.resolve() {
    Ok(_) => (),
    Err(e) => return e.to_compile_error().into(),
  }

  let tokens: proc_macro2::TokenStream = parsed.to_token_stream();
  tokens.into()
}

#[proc_macro]
pub fn component_from_env(tokens: TokenStream) -> TokenStream {
  let mut parsed = match syn::parse::<EnvDescribed>(tokens) {
    Ok(component) => component,
    Err(e) => return e.to_compile_error().into(),
  };

  match parsed.resolve() {
    Ok(_) => (),
    Err(e) => return e.to_compile_error().into(),
  }

  let tokens: proc_macro2::TokenStream = parsed.to_token_stream();
  tokens.into()
}

/// Parses tokens provided, expecting a Component to be the first token(s),
/// followed by additional data parsed by the generic type E.
struct TokenParser<E>
where
  E: syn::parse::Parse + Into<HashMap<String, Library>>,
{
  pub component: Component,
  pub impl_map: HashMap<String, Library>,
  _e: std::marker::PhantomData<E>,
}

impl <E> TokenParser<E>
where
  E: syn::parse::Parse + Into<HashMap<String, Library>>,
{
  fn resolve(&mut self) -> syn::Result<()> {
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
}

impl <E> syn::parse::Parse for TokenParser<E>
where
  E: syn::parse::Parse + Into<HashMap<String, Library>>,
{
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let component = input.parse::<Component>()?;
    let impl_map = input.parse::<E>()?.into();
    Ok(TokenParser { component, impl_map, _e: std::marker::PhantomData })
  }
}

impl <E> quote::ToTokens for TokenParser<E>
where
  E: syn::parse::Parse + Into<HashMap<String, Library>>,
{
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let name = &self.component.name;

    let mut library_list = vec![];
    for library in &self.component.library_list {
      let lib = self.impl_map.get(&library.to_string()).unwrap();
      library_list.push(lib);
    }

    tokens.extend(quote! {
      #name<#(#library_list),*>
    });
  }
}

#[derive(Debug, PartialEq)]
struct Component {
    name: Ident,
    library_list: Vec<Ident>,
}

impl syn::parse::Parse for Component {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
      let name: syn::Ident = input.parse()?;

      input.parse::<Token![<]>()?;

      let library_list: Punctuated<Ident, Comma> = input.call(Punctuated::parse_separated_nonempty)?;
      let library_list: Vec<Ident> = library_list.into_iter().collect();
      input.parse::<Token![>]>()?;
      input.parse::<Token![;]>()?;
      
      Ok(Component {
          name,
          library_list,
      })
  }
}

#[derive(Clone)]
struct Library {
  name: Ident,
  instance: syn::PatPath,
  required: Vec<Ident>,
  resolved: Vec<Library>,
}

impl PartialEq for Library {
  fn eq(&self, other: &Self) -> bool {
    self.name == other.name
      && self.required == other.required
      && self.resolved == other.resolved
  }
}

impl Library {
  /// Attempts to resolve the required libraries for this Library
  fn resolve(&mut self, impl_map: &HashMap<String, Library>) -> syn::Result<()> {
    for required in &self.required {
      if let Some(lib) = impl_map.get(&required.to_string()) {
        if lib.is_resolved() {
          self.resolved.push(lib.clone());
        }
      } else {
        return Err(syn::Error::new(required.span(), format!("Library {} not found", required.to_string())));
      }
    }
    Ok(())
  }

  fn is_resolved(&self) -> bool {
    self.resolved.len() == self.required.len()
  }
}

impl syn::parse::Parse for Library {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let name: syn::Ident = input.parse()?;
    input.parse::<Token![=]>()?;
    let instance = input.parse::<syn::PatPath>()?;
    
    // The Library itself has no required libraries
    if input.is_empty() || input.peek(Token![;]) {
      return Ok(Library {
        name,
        instance,
        required: vec![],
        resolved: vec![],
      });
    }

    // Parse all libraries, which should be a comma separated list between < and >
    input.parse::<Token![<]>()?;
    let required: Punctuated<Ident, Comma>
      = input.call(Punctuated::parse_separated_nonempty)?;
    let required: Vec<Ident> = required.into_iter().collect();
    input.parse::<Token![>]>()?;

    Ok(Library {
      name,
      instance,
      required,
      resolved: vec![],
    })
  }
}

impl quote::ToTokens for Library {

  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let instance = &self.instance;
    let required = &self.resolved;

    let t_tokens = if required.is_empty() {
      quote! {#instance}
    } else {
      quote! {#instance<#(#required),*>}
    };
    tokens.extend(t_tokens);
  }
}

impl std::fmt::Debug for Library {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let instance = self.instance.to_token_stream().to_string();
    write!(
      f, "Library {{ name: {}, instance: {}, required: {:?}, resolved: {:?} }}",
      self.name, instance, self.required, self.resolved
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use quote::quote;
  use proc_macro2;
  use quote::ToTokens;
  use syn::parse_quote;

  #[test]
  fn test_library_parse1() {
    let input = quote! {
      DebugLib=DebugLibBase
    };
    let expected = quote!(DebugLibBase);
  
    let parsed = syn::parse2::<Library>(input).unwrap();
    let parsed: proc_macro2::TokenStream = parsed.to_token_stream();
    assert_eq!(parsed.to_string(), expected.to_string());
  }

  #[test]
  fn test_library_parse2() {
    let input = quote! {
      DebugLib=DebugLibBase<PrintLib>
    };
    let expected = quote!(DebugLibBase<PrintLibBase>);
    let mut parsed = syn::parse2::<Library>(input).unwrap();
    // Manually resolve for testing purposes
    parsed.resolved = vec![Library {
      name: parse_quote!(PrintLib),
      instance: parse_quote!(PrintLibBase),
      required: vec![],
      resolved: vec![]
    }];

    let parsed: proc_macro2::TokenStream = parsed.to_token_stream();
    assert_eq!(parsed.to_string(), expected.to_string());
  }

  #[test]
  fn test_library_parse3() {
    let input = quote! {
      DebugLib=DebugLibBase<PrintLib, WriteLib>
    };
    let expected = quote!(DebugLibBase<PrintLibBase, WriteLibBase>);
    let mut parsed = syn::parse2::<Library>(input).unwrap();
    // Manually resolve for testing purposes
    parsed.resolved = vec![
      Library {
        name: parse_quote!(PrintLib),
        instance: parse_quote!(PrintLibBase),
        required: vec![],
        resolved: vec![]
      },
      Library {
        name: parse_quote!(WriteLib),
        instance: parse_quote!(WriteLibBase),
        required: vec![],
        resolved: vec![]
      }
    ];

    let parsed: proc_macro2::TokenStream = parsed.to_token_stream();
    assert_eq!(parsed.to_string(), expected.to_string());
  }

  #[test]
  fn test_library_parse4() {
    let input = quote! {
      DebugLib=DebugLibBase<PrintLib, WriteLib>
    };
    let expected = quote!(DebugLibBase<PrintLibBase<WriteLibBase>, WriteLibBase>);
    let mut parsed = syn::parse2::<Library>(input).unwrap();
    parsed.resolved = vec![
      Library {
        name: parse_quote!(PrintLib),
        instance: parse_quote!(PrintLibBase),
        required: vec![parse_quote!(WriteLib)],
        resolved: vec![Library {
          name: parse_quote!(WriteLib),
          instance: parse_quote!(WriteLibBase),
          required: vec![],
          resolved: vec![]
        }]
      },
      Library {
        name: parse_quote!(WriteLib),
        instance: parse_quote!(WriteLibBase),
        required: vec![],
        resolved: vec![]
      }
    ];
    let parsed: proc_macro2::TokenStream = parsed.to_token_stream();
    assert_eq!(parsed.to_string(), expected.to_string());
  }

  #[test]
  fn test_library_parse5() {
    let input = quote! {
      DebugLib=pkg1::library::DebugLibBase<PrintLib, WriteLib>
    };
    let expected = quote!(
      pkg1::library::DebugLibBase<pkg1::library::PrintLibBase<pkg1::library::WriteLibBase>, pkg1::library::WriteLibBase>
    );
    let mut parsed = syn::parse2::<Library>(input).unwrap();
    parsed.resolved = vec![
      Library {
        name: parse_quote!(PrintLib),
        instance: parse_quote!(pkg1::library::PrintLibBase),
        required: vec![parse_quote!(WriteLib)],
        resolved: vec![Library {
          name: parse_quote!(WriteLib),
          instance: parse_quote!(pkg1::library::WriteLibBase),
          required: vec![],
          resolved: vec![]
        }]
      },
      Library {
        name: parse_quote!(WriteLib),
        instance: parse_quote!(pkg1::library::WriteLibBase),
        required: vec![],
        resolved: vec![]
      }
    ];
    let parsed: proc_macro2::TokenStream = parsed.to_token_stream();
    assert_eq!(parsed.to_string(), expected.to_string());
  }

  #[test]
  fn test_full_parse1() {
    let expected = quote! {
      MyDriver<DebugLibBase>
    };

    let input = quote! {
      MyDriver<DebugLib>; DebugLib=DebugLibBase
    };

    let mut parsed = syn::parse2::<FullyDescribed>(input).unwrap();
    parsed.resolve().unwrap();
    let parsed = parsed.to_token_stream();
    assert_eq!(parsed.to_string(), expected.to_string());
  }

  #[test]
  fn test_full_parse2() {
    let expected_output = quote! {
      MyDriver < DebugLibBase < PrintLibBase > >
    };

    let input = quote! {
      MyDriver<DebugLib>;
      DebugLib=DebugLibBase<PrintLib>;
      PrintLib=PrintLibBase;
    };

    let mut parsed = syn::parse2::<FullyDescribed>(input).unwrap();
    parsed.resolve().unwrap();
    let parsed = parsed.to_token_stream();
    assert_eq!(parsed.to_string(), expected_output.to_string());
  }

  #[test]
  fn test_full_parse3() {
    let expected_output = quote! {
      MyDriver < DebugLibBase < PrintLibBase >, AdvLibBase < PrintLibBase> >
    };

    let input = quote! {
      MyDriver<DebugLib, AdvLib>;
      DebugLib=DebugLibBase<PrintLib>;
      AdvLib=AdvLibBase<PrintLib>;
      PrintLib=PrintLibBase;
    };
  
    let mut parsed = syn::parse2::<FullyDescribed>(input).unwrap();
    parsed.resolve().unwrap();
    let parsed = parsed.to_token_stream();
    assert_eq!(parsed.to_string(), expected_output.to_string());
  }

  #[test]
  fn test_full_parse4() {
    let expected_output = quote! {
      MyDriver < DebugLibBase < PrintLibBase >, AdvLibBase < PrintLibBase, WriteLibBase > >
    };

    let input = quote! {
      MyDriver<DebugLib, AdvLib>;
      DebugLib=DebugLibBase<PrintLib>;
      AdvLib=AdvLibBase<PrintLib, WriteLib>;
      PrintLib=PrintLibBase;
      WriteLib=WriteLibBase;
    };

    let mut parsed = syn::parse2::<FullyDescribed>(input).unwrap();
    parsed.resolve().unwrap();
    let parsed = parsed.to_token_stream();
    assert_eq!(parsed.to_string(), expected_output.to_string());
  }

  #[test]
  /// Test that the config file can be used.
  fn test_full_parse5() {
    let expected_output = quote! {
      MyDriver < DebugLibBase < PrintLibBase >, AdvLibBase < PrintLibBase, WriteLibSpecial > >
    };

    let input = quote! {
      MyDriver<DebugLib, AdvLib>;
      Path = "tests/data/test_config.toml";
    };

    let mut parsed = syn::parse2::<PathDescribed>(input).unwrap();
    parsed.resolve().unwrap();
    let parsed = parsed.to_token_stream();
    assert_eq!(parsed.to_string(), expected_output.to_string());
  }

  #[test]
  /// Test that the config file parser can properly handle include paths
  fn test_full_parse6() {
    let expected_output = quote! {
      MyDriver < pk1::library::DebugLibBase, pk1::library::AdvLibSpecial < pk2::library::WriteLibBase, pk1::library::DebugLibBase > >
    };

    let input = quote! {
      MyDriver<DebugLib, AdvLib>;
      DebugLib=pk1::library::DebugLibBase;
      AdvLib=pk1::library::AdvLibSpecial < WriteLib, DebugLib >;
      WriteLib=pk2::library::WriteLibBase; 
    };

    let mut parsed = syn::parse2::<FullyDescribed>(input).unwrap();
    parsed.resolve().unwrap();
    let parsed = parsed.to_token_stream();
    assert_eq!(parsed.to_string(), expected_output.to_string());
  }
}