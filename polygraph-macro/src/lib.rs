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

fn lifetime_a() -> syn::Generics {
    let mut params = syn::punctuated::Punctuated::new();
    params.push(syn::GenericParam::Lifetime(syn::LifetimeDef {
        attrs: Vec::new(),
        lifetime: syn::Lifetime {
            apostrophe: proc_macro2::Span::call_site(),
            ident: quote::format_ident!("a"),
        },
        colon_token: None,
        bounds: syn::punctuated::Punctuated::new(),
    }));
    syn::Generics {
        lt_token: Some(syn::Token![<](proc_macro2::Span::call_site())),
        params,
        gt_token: Some(syn::Token![>](proc_macro2::Span::call_site())),
        where_clause: None,
    }
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
        let table_structs: Vec<_> = self.structs.iter().map(|x| {
            let mut x = x.clone();
            x.vis = syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::Token!(pub)(x.span())
            });
            // x.generics = lifetime_a();
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
        let table_enums: Vec<_> = self.enums.iter().map(|x| {
            let mut x = x.clone();
            x.vis = syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::Token!(pub)(x.span())
            });
            x
        }).collect();
        SchemaOutput {
            name: self.name.clone(),
            save_structs,
            table_structs,
            save_enums,
            table_enums,
        }
    }
}

#[proc_macro]
pub fn schema(raw_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use heck::ShoutySnakeCase;
    use heck::SnakeCase;

    let input: SchemaInput = syn::parse_macro_input!(raw_input as SchemaInput);
    println!("input is {:#?}", input);
    let output = input.process();
    let save_structs = output.save_structs.iter();
    let mut save_names: Vec<_> =
        output.save_structs.iter().map(|x| x.ident.clone()).collect();
    let table_structs = output.table_structs.iter();
    let table_names: Vec<_> =
        output.table_structs.iter()
        .map(|x| quote::format_ident!("{}", x.ident.to_string().to_snake_case()))
        .collect();
    let table_inserts: Vec<_> =
        output.table_structs.iter()
        .map(|x| quote::format_ident!("insert_{}", x.ident.to_string().to_snake_case()))
        .collect();
    let table_types: Vec<_> =
        output.table_structs.iter()
        .map(|x| x.ident.clone())
        .collect();
    let save_enums = output.save_enums.iter();
    let table_enums = output.table_enums.iter();
    save_names.extend(
        output.save_enums.iter().map(|x| x.ident.clone()));
    let name = &input.name;
    let savename = quote::format_ident!("{}Save", name);
    let internalname = quote::format_ident!("Internal{}", name);
    let keys = quote::format_ident!("KEYS_{}", name.to_string().to_shouty_snake_case());
    let output = quote::quote!{
        #(
            #save_structs
        )*
        #(
            #table_structs
        )*
        #(
            #save_enums
        )*
        #(
            #table_enums
        )*
        #[allow(non_snake_case)]
        pub struct #savename(
            #(
                pub Vec<#save_names>
            ),*
        );

        #[allow(non_snake_case)]
        pub struct #internalname {
            #(
                pub #table_names: Vec<#table_types>,
            )*
        }
        impl #internalname {
            fn new() -> Self {
                #internalname {
                    #( #table_names: Vec::new(), )*
                }
            }
        }

        lazy_static::lazy_static! {
            static ref #keys: std::sync::Mutex<std::collections::HashMap<std::any::TypeId,#internalname>>
                = std::sync::Mutex::new(std::collections::HashMap::new());
        }

        #[derive(Clone,Copy,Debug)]
        pub struct #name<K>(std::marker::PhantomData<K>);

        impl<K: 'static> #name<K> {
            // pub fn open(_path: &str) -> Result<Self, String> {
            //     let type_id = std::any::TypeId::of::<K>();
            //     let mut keys = #keys.lock().unwrap();
            //     if keys.contains_key(&type_id) {
            //         Err("Key type already used in another DB".to_string())
            //     } else {
            //         keys.insert(type_id, #inernalname::new());
            //         Ok(#name(std::marker::PhantomData))
            //     }
            // }
            /// Create an empty #name database.
            pub fn new() -> Result<Self, String> {
                let type_id = std::any::TypeId::of::<K>();
                let mut keys = #keys.lock().unwrap();
                if keys.contains_key(&type_id) {
                    Err("Key type already used in another DB".to_string())
                } else {
                    keys.insert(type_id, #internalname::new());
                    Ok(#name(std::marker::PhantomData))
                }
            }
        }

        pub struct Key<'a,K: 'a, T: 'a>(usize, std::marker::PhantomData<&'a (K,T)>);

        impl<K: 'static> #name<K> {
            #(
                pub fn #table_inserts(&self, datum: #table_types) -> Key<K, #table_types> {
                    // peek::<K,Key<K,Foo>>(|i| {
                    // })
                    let type_id = std::any::TypeId::of::<K>();
                    let mut keys = #keys.lock().unwrap();
                    if let Some(i) = keys.get_mut(&type_id) {
                        i.#table_names.push(datum);
                        Key(i.#table_names.len(), std::marker::PhantomData)
                    } else {
                        unreachable!()
                    }
                }
            )*
        }

        fn peek<K: 'static, T>(f: impl Fn(&#internalname) -> T) -> T {
            let type_id = std::any::TypeId::of::<K>();
            let mut keys = #keys.lock().unwrap();
            if let Some(i) = keys.get(&type_id) {
                f(i)
            } else {
                panic!("There is an invalid K")
            }
        }
        impl<'a, K: 'static> Key<'a,K,Foo> {
            pub fn get(&self) -> &Foo {
                let type_id = std::any::TypeId::of::<K>();
                let mut keys = #keys.lock().unwrap();
                if let Some(i) = keys.get_mut(&type_id) {
                    // The following unsafe code is sound because we
                    // do not allow any mutable borrows when there are
                    // keys out.
                    unsafe { std::mem::transmute(&i.foo[self.0]) }
                } else {
                    unreachable!()
                }
                // let type_id = TypeId::of::<K>();
                // let mut keys = #keys.lock().unwrap();
                // if let Some(DatabaseInternal(ref v)) = keys.get_mut(&type_id) {
                //     v[self.0-1].clone()
                // } else {
                //     unreachable!()
                // }
            }
        }
    };
    println!("output is {}", output.to_string());
    output.into()
}
