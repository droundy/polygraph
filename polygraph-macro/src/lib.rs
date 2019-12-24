// Copyright 2019 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the GPL version 2.0 or later.

//! This crate defines a macro for a database.

extern crate proc_macro;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

#[derive(Debug)]
struct SchemaInput {
    items: Vec<syn::Item>,
}

impl syn::parse::Parse for SchemaInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(SchemaInput {
            items: {
                let mut items = Vec::new();
                while !input.is_empty() {
                    items.push(input.parse()?);
                }
                items
            },
        })
    }
}

#[proc_macro]
pub fn schema(raw_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: SchemaInput = syn::parse_macro_input!(raw_input as SchemaInput);
    println!("input is {:?}", input);
    "fn answer() -> u32 { 42 }".parse().unwrap()
}
