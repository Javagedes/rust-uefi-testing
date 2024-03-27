use super::{Component, Library};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Token;

use std::collections::HashMap;


pub fn parse(tokens: TokenStream) -> TokenStream {
    let mut parsed = match syn::parse2::<FullyDescribed>(tokens) {
        Ok(component) => component,
        Err(e) => return e.to_compile_error().into(),
    };

    match parsed.resolve() {
        Ok(_) => (),
        Err(e) => return e.to_compile_error().into(),
    }

    parsed.to_token_stream()
}

struct FullyDescribed {
    component: Component,
    impl_map: HashMap<String, Library>,
}

impl FullyDescribed {
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

impl syn::parse::Parse for FullyDescribed {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let component = input.parse::<Component>()?;

        let impl_map = input.parse_terminated(Library::parse, Token![;])?;
        let impl_map: HashMap<String, Library> = impl_map
            .into_iter()
            .map(|lib| (lib.name.to_string().to_lowercase(), lib))
            .collect();

        Ok(FullyDescribed { component, impl_map })
    }
}

impl ToTokens for FullyDescribed {
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
            MyDriver<DebugLib>; DebugLib=DebugLibBase
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
            DebugLib=DebugLibBase<PrintLib>;
            PrintLib=PrintLibBase;
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
            DebugLib=DebugLibBase<PrintLib>;
            AdvLib=AdvLibBase<PrintLib>;
            PrintLib=PrintLibBase;
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
            DebugLib=DebugLibBase<PrintLib>;
            AdvLib=AdvLibBase<PrintLib, WriteLib>;
            PrintLib=PrintLibBase;
            WriteLib=WriteLibBase;
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
            DebugLib=pk1::library::DebugLibBase;
            AdvLib=pk1::library::AdvLibSpecial < WriteLib, DebugLib >;
            WriteLib=pk2::library::WriteLibBase; 
        };

        let actual = parse(input);
        assert_eq!(actual.to_string(), expected_output.to_string());
    }
}
