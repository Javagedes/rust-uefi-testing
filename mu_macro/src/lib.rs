extern crate proc_macro;

mod from_macro;
mod from_path;

use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, token::Comma, Ident, Token};

#[proc_macro]
pub fn component(tokens: TokenStream) -> TokenStream {
    from_macro::parse(tokens.into()).into()
}

#[proc_macro]
pub fn component_from_path(tokens: TokenStream) -> TokenStream {
  from_path::parse(tokens.into()).into()
}

#[derive(Debug, PartialEq, Clone)]
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
      if let Some(lib) = impl_map.get(&required.to_string().to_lowercase()) {
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
}