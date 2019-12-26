// Copyright 2019 David Roundy <roundyd@physics.oregonstate.edu>
//
// Licensed under the GPL version 2.0 or later.

//! This crate defines a macro for a database.

extern crate proc_macro;

use syn::spanned::Spanned;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}


enum Item {
    Struct(syn::ItemStruct),
    Enum(syn::ItemEnum),
}

impl syn::parse::Parse for Item {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attrs = input.call(syn::Attribute::parse_outer)?;
        let ahead = input.fork();
        let vis: syn::Visibility = ahead.parse()?;

        let lookahead = ahead.lookahead1();
        let mut item =
            if lookahead.peek(syn::Token![struct]) {
                input.parse().map(Item::Struct)
            } else if lookahead.peek(syn::Token![enum]) {
                input.parse().map(Item::Enum)
            } else {
                Err(lookahead.error())
            }?;

        {
            let (item_vis, item_attrs, generics) = match &mut item {
                Item::Struct(item) => (&mut item.vis, &mut item.attrs, &item.generics),
                Item::Enum(item) => (&mut item.vis, &mut item.attrs, &item.generics),
            };
            if generics.params.len() > 0 {
                return Err(syn::Error::new_spanned(generics,
                                                   "schema! does not support generic types."));
            }
            attrs.extend(item_attrs.drain(..));
            *item_attrs = attrs;
            *item_vis = vis;
        }

        Ok(item)
    }
}

#[derive(Debug)]
struct SchemaInput {
    name: syn::Ident,
    structs: Vec<syn::ItemStruct>,
    enums: Vec<syn::ItemEnum>,
}

impl syn::parse::Parse for SchemaInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![type]>()?;
        let name: syn::Ident = input.parse()?;
        input.parse::<syn::Token![;]>()?;
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        while !input.is_empty() {
            match input.parse()? {
                Item::Struct(i) => structs.push(i),
                Item::Enum(i) => enums.push(i),
            }
        }
        Ok(SchemaInput {
            name,
            structs,
            enums,
        })
    }
}

#[derive(Debug)]
struct SchemaOutput {
    name: syn::Ident,
    save_structs: Vec<syn::ItemStruct>,
    save_enums: Vec<syn::ItemEnum>,
    view_structs: Vec<syn::ItemStruct>,
    view_enums: Vec<syn::ItemEnum>,
}

impl SchemaInput {
    fn process(&self) -> SchemaOutput {
        let mut tables = std::collections::HashSet::new();
        tables.extend(self.structs.iter().map(|x| x.ident.clone()));
        tables.extend(self.enums.iter().map(|x| x.ident.clone()));

        let save_structs: Vec<_> = self.structs.iter().map(|x| {
            let mut x = x.clone();
            x.vis = syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::Token!(pub)(x.span())
            });
            x.ident = syn::Ident::new(&format!("Save{}", x.ident.to_string()), x.ident.span());
            x
        }).collect();
        let view_structs: Vec<_> = self.structs.iter().map(|x| {
            let mut x = x.clone();
            x.vis = syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::Token!(pub)(x.span())
            });
            x
        }).collect();

        let save_enums: Vec<_> = self.enums.iter().map(|x| {
            let mut x = x.clone();
            x.vis = syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::Token!(pub)(x.span())
            });
            x.ident = syn::Ident::new(&format!("Save{}", x.ident.to_string()), x.ident.span());
            x
        }).collect();
        let view_enums: Vec<_> = self.enums.iter().map(|x| {
            let mut x = x.clone();
            x.vis = syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::Token!(pub)(x.span())
            });
            x
        }).collect();
        SchemaOutput {
            name: self.name.clone(),
            save_structs,
            view_structs,
            save_enums,
            view_enums,
        }
    }
}

#[proc_macro]
pub fn schema(raw_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: SchemaInput = syn::parse_macro_input!(raw_input as SchemaInput);
    println!("input is {:#?}", input);
    let output = input.process();
    let save_structs = output.save_structs.iter();
    let mut save_names: Vec<_> =
        output.save_structs.iter().map(|x| x.ident.clone()).collect();
    let view_structs = output.view_structs.iter();
    let save_enums = output.save_enums.iter();
    let view_enums = output.view_enums.iter();
    save_names.extend(
        output.save_enums.iter().map(|x| x.ident.clone()));
    let name = &input.name;
    let savename = quote::format_ident!("{}Save", name);
    let output = quote::quote!{
        #(
            #save_structs
        )*
        #(
            #view_structs
        )*
        #(
            #save_enums
        )*
        #(
            #view_enums
        )*
        #[allow(non_snake_case)]
        pub struct #savename(
            #(
                pub Vec<#save_names>
            ),*
        );

        #[allow(non_snake_case)]
        pub struct #name {
            #(
                pub #save_names: Vec<#save_names>,
            )*
        }
        impl #name {
            fn new() -> Self {
                #name {
                    #( #save_names: Vec::new(), )*
                }
            }
        }
        macro_rules! mkschema {
            () => {{
                let mut internal_data = #name::new();
                
            }}
        }
    };
    println!("output is {}", output.to_string());
    output.into()
}
