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
    table_structs: Vec<syn::ItemStruct>,
    table_enums: Vec<syn::ItemEnum>,
}

// fn lifetime_a() -> syn::Generics {
//     let mut params = syn::punctuated::Punctuated::new();
//     params.push(syn::GenericParam::Lifetime(syn::LifetimeDef {
//         attrs: Vec::new(),
//         lifetime: syn::Lifetime {
//             apostrophe: proc_macro2::Span::call_site(),
//             ident: quote::format_ident!("a"),
//         },
//         colon_token: None,
//         bounds: syn::punctuated::Punctuated::new(),
//     }));
//     syn::Generics {
//         lt_token: Some(syn::Token![<](proc_macro2::Span::call_site())),
//         params,
//         gt_token: Some(syn::Token![>](proc_macro2::Span::call_site())),
//         where_clause: None,
//     }
// }

#[derive(Debug,Eq,PartialEq)]
enum KeyType {
    OptionKey(syn::Ident),
}

fn parse_keytype(t: &mut syn::Type) -> Result<Option<KeyType>, syn::Error> {
    if let syn::Type::Path(p) = t {
        let path_count = p.path.segments.len();
        println!("path is {:#?}", p);
        println!("path_count is {:#?}", path_count);
        if path_count == 1 {
            let ident = p.path.segments.last().unwrap().clone().ident;
            let path_only = p.path.segments.last_mut().unwrap();
            let name = ident.to_string();
            println!("path_only is {:#?}", name);
            if name == "Option" {
                let args = path_only.clone().arguments;
                println!("args are {:#?}", args);
                unimplemented!()
            } else {
                if name == "Key" {
                    if let syn::PathArguments::AngleBracketed(args) = &mut path_only.arguments {
                        if args.args.len() != 1 {
                            return Err(syn::Error::new_spanned(
                                t,
                                "Key should have just one type argument")
                            );
                        }
                        use syn::{GenericArgument, Type};
                        if let GenericArgument::Type(Type::Path(ap)) = args.args.first().unwrap() {
                            if ap.path.segments.len() != 1 {
                                return Err(syn::Error::new_spanned(
                                    t,
                                    "Key should have a simple type argument")
                                );
                            }
                            let tp = ap.path.segments.first().unwrap();
                            if !tp.arguments.is_empty() {
                                Err(syn::Error::new_spanned(tp.arguments.clone(),
                                                            "Key type should be a simple table name"))
                            } else {
                                let i = tp.ident.clone();
                                args.args = [syn::parse_quote!{K},
                                             args.args.first().unwrap().clone()]
                                    .into_iter().cloned().collect();
                                println!("new args: {:?}", args.args);
                                Ok(Some(KeyType::OptionKey(i)))
                            }
                        } else {
                            Err(syn::Error::new_spanned(
                                t,
                                "Key should have a simple type argument")
                            )
                        }
                    } else {
                        Err(syn::Error::new_spanned(
                            t,
                            "Key should be Key<ATableType>")
                        )
                    }
                } else {
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

fn parse_fields(f: &mut syn::FieldsNamed)
                -> Result<std::collections::HashMap<syn::Ident,KeyType>, syn::Error>
{
    let mut keymap = std::collections::HashMap::new();
    for n in f.named.iter_mut() {
        if let Some(kt) = parse_keytype(&mut n.ty)? {
            keymap.insert(n.ident.clone().unwrap(), kt);
        }
    }
    Ok(keymap)
}

impl SchemaInput {
    fn process(&self) -> Result<SchemaOutput, syn::Error> {
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
        let mut table_structs = Vec::with_capacity(self.structs.len());
        for mut x in self.structs.iter().cloned() {
            x.vis = syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::Token!(pub)(x.span())
            });
            match &mut x.fields {
                syn::Fields::Named(n) => {
                    let keymap = parse_fields(n)?;
                    if keymap.len() > 0 {
                        // We have keys in here, so we will need this
                        // struct to be generic over database K.
                        x.generics = syn::parse_quote!{<K>}
                    }
                    for (f,k) in keymap.iter() {
                        println!("{}: {:?}", f.to_string(), k);
                        // panic!("what to do now? {}: {:?}", f.to_string(), k);
                    }
                }
                syn::Fields::Unnamed(_) => {
                }
                syn::Fields::Unit => {
                    // Nothing to do for this
                }
            }
            // x.generics = lifetime_a();
            table_structs.push(x)
        }

        let save_enums: Vec<_> = self.enums.iter().map(|x| {
            let mut x = x.clone();
            x.vis = syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::Token!(pub)(x.span())
            });
            x.ident = syn::Ident::new(&format!("Save{}", x.ident.to_string()), x.ident.span());
            x
        }).collect();
        let table_enums: Vec<_> = self.enums.iter().map(|x| {
            let mut x = x.clone();
            x.vis = syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::Token!(pub)(x.span())
            });
            x
        }).collect();
        Ok(SchemaOutput {
            name: self.name.clone(),
            save_structs,
            table_structs,
            save_enums,
            table_enums,
        })
    }
}

#[proc_macro]
pub fn schema(raw_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // use heck::ShoutySnakeCase;
    use heck::SnakeCase;

    let input: SchemaInput = syn::parse_macro_input!(raw_input as SchemaInput);
    println!("input is {:#?}", input);
    let output = match input.process() {
        Err(e) => {
            return e.to_compile_error().into();
        }
        Ok(v) => v,
    };
    // let save_structs = output.save_structs.iter();
    // let mut save_names: Vec<_> =
    //     output.save_structs.iter().map(|x| x.ident.clone()).collect();
    let table_structs = output.table_structs.iter();
    let table_names: Vec<_> =
        output.table_structs.iter()
        .map(|x| quote::format_ident!("{}", x.ident.to_string().to_snake_case()))
        .collect();
    let table_inserts: Vec<_> =
        output.table_structs.iter()
        .map(|x| quote::format_ident!("insert_{}", x.ident.to_string().to_snake_case()))
        .collect();
    let table_lookups: Vec<_> =
        output.table_structs.iter()
        // only allow lookups on non-generic fields
        .filter(|x| x.generics.params.len() == 0)
        .map(|x| quote::format_ident!("lookup_{}", x.ident.to_string().to_snake_case()))
        .collect();
    let table_lookup_hashes: Vec<_> =
        output.table_structs.iter()
        // only allow lookups on non-generic fields
        .filter(|x| x.generics.params.len() == 0)
        .map(|x| quote::format_ident!("hash_{}", x.ident.to_string().to_snake_case()))
        .collect();
    let table_types: Vec<syn::PathSegment> =
        output.table_structs.iter()
        .map(|x| {
            let i = x.ident.clone();
            let g = x.generics.clone();
            syn::parse_quote!{#i#g}
        })
        .collect();
    // let save_enums = output.save_enums.iter();
    let table_enums = output.table_enums.iter();
    // save_names.extend(
    //     output.save_enums.iter().map(|x| x.ident.clone()));
    let name = &input.name;
    // let savename = quote::format_ident!("{}Save", name);
    let output = quote::quote!{
        // #(
        //     #save_structs
        // )*
        // #(
        //     #save_enums
        // )*
        #(
            #[derive(Eq,PartialEq,Hash,Clone)]
            #table_structs
        )*
        #(
            #[derive(Eq,PartialEq,Hash,Clone)]
            #table_enums
        )*
        // #[allow(non_snake_case)]
        // pub struct #savename(
        //     #(
        //         pub Vec<#save_names>
        //     ),*
        // );

        #[allow(non_snake_case)]
        pub struct #name<K> {
            #(
                pub #table_names: Vec<#table_types>,
            )*
            #(
                pub #table_lookup_hashes: std::collections::HashMap<#table_types, usize>,
            )*
            phantom: std::marker::PhantomData<K>,
        }
        impl<K: Fn()> #name<K> {
            /// Create an empty #name database.
            pub fn new(_: K) -> Self {
                #name {
                    #( #table_names: Vec::new(), )*
                    #(
                        #table_lookup_hashes: std::collections::HashMap::new(),
                    )*
                    phantom: std::marker::PhantomData,
                }
            }
        }

        #[derive(Clone,Copy,Eq,PartialEq,Hash)]
        pub struct Key<K, T>(usize, std::marker::PhantomData<(K,T)>);

        impl<K: 'static> #name<K> {
            #(
                pub fn #table_inserts(&mut self, datum: #table_types) -> Key<K, #table_types> {
                    let idx = self.#table_names.len();
                    self.#table_names.push(datum.clone());
                    self.#table_lookup_hashes.insert(datum, idx);
                    Key(idx, std::marker::PhantomData)
                }
            )*
            #(
                pub fn #table_lookups(&self, datum: &#table_types) -> Option<Key<K, #table_types>> {
                    self.#table_lookup_hashes.get(datum)
                        .map(|&i| Key(i, std::marker::PhantomData))
                    // self.0.#table_names.iter().enumerate()
                    //     .filter(|&(_,x)| x == datum)
                    //     .map(|(i,x)| Key(i, std::marker::PhantomData))
                    //     .next()
                }
            )*
        }
        #(
            impl<K> Key<K,#table_types> {
                pub fn d<'a,'b>(&'a self, database: &'b #name<K>) -> &'b #table_types {
                    &database.#table_names[self.0]
                }
            }
        )*
    };
    println!("\n\n\noutput is\n\n{}", output.to_string());
    output.into()
}
