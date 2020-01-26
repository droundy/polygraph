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
    pod_structs: Vec<syn::ItemStruct>,
    pod_enums: Vec<syn::ItemEnum>,
    key_structs: Vec<syn::ItemStruct>,
    key_struct_maps: Vec<std::collections::HashMap<syn::Ident, KeyType>>,
    key_enums: Vec<syn::ItemEnum>,
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
    Key(syn::Ident),
    OptionKey(syn::Ident),
}

impl KeyType {
    fn key_to(&self) -> syn::Ident {
        match self {
            KeyType::Key(i) => i.clone(),
            KeyType::OptionKey(i) => i.clone(),
        }
    }
}

fn first_of_type(t: &syn::Type) -> Option<(syn::Ident, syn::Type)> {
    let p = if let syn::Type::Path(p) = t {
        p
    } else {
        return None;
    };
    let path_count = p.path.segments.len();
    if path_count != 1 {
        return None;
    }
    let ident = p.path.segments.last().unwrap().clone().ident;
    let path_only = p.path.segments.last().unwrap();
    let args = if let syn::PathArguments::AngleBracketed(args) = &path_only.arguments {
        args
    } else {
        return None;
    };
    if args.args.len() != 1 {
        return None;
    }
    use syn::{GenericArgument};
    let t = if let GenericArgument::Type(t) = args.args.first()? {
        t
    } else {
        return None;
    };
    Some((ident, t.clone()))
}

fn type_is_just_ident(t: &syn::Type) -> Option<syn::Ident> {
    let p = if let syn::Type::Path(p) = t {
        p
    } else {
        return None;
    };
    let path_count = p.path.segments.len();
    if path_count != 1 {
        return None;
    }
    let ident = p.path.segments.last().unwrap().clone().ident;
    let path_only = p.path.segments.last().unwrap();
    if path_only.arguments != syn::PathArguments::None {
        return None;
    }
    Some(ident)
}

fn parse_keytype(t: &syn::Type) -> Result<Option<KeyType>, syn::Error> {
    if let Some((key, t)) = first_of_type(&t) {
        if key.to_string() == "Option" {
            if let Some((key, t)) = first_of_type(&t) {
                if key.to_string() == "Key" {
                    if let Some(i) = type_is_just_ident(&t) {
                        return Ok(Some(KeyType::OptionKey(i)));
                    } else {
                        return Err(syn::Error::new_spanned(t,
                                                           "Key type should be a simple table name"));
                    }
                }
            }
        }
    }
    if let syn::Type::Path(p) = t {
        let path_count = p.path.segments.len();
        println!("path is {:#?}", p);
        println!("path_count is {:#?}", path_count);
        if path_count == 1 {
            let ident = p.path.segments.last().unwrap().clone().ident;
            let path_only = p.path.segments.last().unwrap();
            let name = ident.to_string();
            println!("path_only is {:#?}", name);
            if name == "Option" {
                let args = path_only.clone().arguments;
                println!("args are {:#?}", args);
                unimplemented!()
            } else {
                if name == "Key" {
                    if let syn::PathArguments::AngleBracketed(args) = &path_only.arguments {
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
                                // args.args = [// syn::parse_quote!{K},
                                //              args.args.first().unwrap().clone()]
                                //     .into_iter().cloned().collect();
                                // println!("new args: {:?}", args.args);
                                Ok(Some(KeyType::Key(i)))
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

fn parse_fields(f: &syn::FieldsNamed)
                -> Result<std::collections::HashMap<syn::Ident,KeyType>, syn::Error>
{
    let mut keymap = std::collections::HashMap::new();
    for n in f.named.iter() {
        if let Some(kt) = parse_keytype(&n.ty)? {
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

        let mut pod_structs = Vec::new();
        let mut key_structs = Vec::new();
        let mut key_struct_maps = Vec::new();

        for x in self.structs.iter().cloned() {
            match &x.fields {
                syn::Fields::Named(n) => {
                    let keymap = parse_fields(n)?;
                    if keymap.len() > 0 {
                        key_struct_maps.push(keymap);
                        key_structs.push(x);
                    } else {
                        pod_structs.push(x);
                    }
                }
                syn::Fields::Unnamed(_) => {
                    pod_structs.push(x);
                }
                syn::Fields::Unit => {
                    pod_structs.push(x);
                }
            }
        }

        let pod_enums: Vec<_> = self.enums.iter().map(|x| {
            let mut x = x.clone();
            x.vis = syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::Token!(pub)(x.span())
            });
            x
        }).collect();
        Ok(SchemaOutput {
            name: self.name.clone(),
            pod_structs,
            key_structs,
            key_struct_maps,
            key_enums: Vec::new(),
            pod_enums,
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

    // Here "pod" means "plain old data", and refers to tables that
    // have no keys in them.  When such tables exist, there is just
    // one possible reason: We want to create a back hash so we can
    // quickly search for all things that reference a given value,
    // which means that we need to effectively intern those values.
    // This also may save size, if the same large value is used many
    // times in the database (essentially interning).
    let pod_structs = &output.pod_structs;
    let key_structs = &output.key_structs;

    let key_names: Vec<_> =
        key_structs.iter()
        .map(|x| quote::format_ident!("{}", x.ident.to_string().to_snake_case()))
        .collect();

    let mut reverse_references = std::collections::HashMap::new();
    for (map,t) in output.key_struct_maps.iter().zip(key_structs.iter()) {
        println!("hello we have {:?}", t);
        for (k,v) in map.iter() {
            let kt = v.key_to();
            if !reverse_references.contains_key(&kt) {
                reverse_references.insert(kt.clone(), Vec::new());
            }
            reverse_references.get_mut(&kt).unwrap().push((t.ident.clone(), k.clone()));
        }
    }
    println!("\n\nreverse references are {:?}", reverse_references);

    let mut pod_query_backrefs: Vec<Vec<(syn::Ident, syn::Ident)>> = Vec::new();
    let pod_query_structs: Vec<syn::ItemStruct> = pod_structs.iter().cloned()
        .map(|mut x| {
            let i = x.ident.clone();
            let mut backrefs = Vec::new();
            let mut backrefs_code = Vec::new();
            if let Some(v) = reverse_references.get(&x.ident) {
                for r in v.iter() {
                    let field = quote::format_ident!("{}_of", r.1.to_string().to_snake_case());
                    let t = &r.0;
                    backrefs.push((t.clone(), field.clone()));
                    let code = quote::quote!{
                        pub #field: KeySet<#t>,
                    };
                    println!("\ncode is {:?}", code.to_string());
                    backrefs_code.push(code);
                }
            }
            pod_query_backrefs.push(backrefs);
            x.ident = quote::format_ident!("{}Query", x.ident);
            x.fields = syn::Fields::Named(syn::parse_quote!{{
                __data: #i,
                #(#backrefs_code)*
            }});
            x
        })
        .collect();
    let pod_query_types: Vec<syn::PathSegment> =
        pod_query_structs.iter()
        .map(|x| {
            let i = x.ident.clone();
            syn::parse_quote!{#i}
        })
        .collect();
    let pod_query_new: Vec<_> =
        pod_query_structs.iter().zip(pod_query_backrefs.iter())
        .map(|(x,br)| {
            let i = &x.ident;
            let backcode = br.iter().map(|(t,f)| {
                quote::quote!{
                    #f: KeySet::<#t>::new(),
                }
            });
            quote::quote!{
                #i {
                    __data: value,
                    #(#backcode)*
                }
            }
        })
        .collect();

    let pod_names: Vec<_> =
        pod_structs.iter()
        .map(|x| quote::format_ident!("{}", x.ident.to_string().to_snake_case()))
        .collect();
    let pod_inserts: Vec<_> =
        pod_structs.iter()
        .map(|x| quote::format_ident!("insert_{}", x.ident.to_string().to_snake_case()))
        .collect();
    let pod_lookups: Vec<_> =
        pod_structs.iter()
        // only allow lookups on non-generic fields
        .filter(|x| x.generics.params.len() == 0)
        .map(|x| quote::format_ident!("lookup_{}", x.ident.to_string().to_snake_case()))
        .collect();
    let pod_lookup_hashes: Vec<_> =
        pod_structs.iter()
        // only allow lookups on non-generic fields
        .filter(|x| x.generics.params.len() == 0)
        .map(|x| quote::format_ident!("hash_{}", x.ident.to_string().to_snake_case()))
        .collect();
    let pod_types: Vec<syn::PathSegment> =
        pod_structs.iter()
        .map(|x| {
            let i = x.ident.clone();
            syn::parse_quote!{#i}
        })
        .collect();

    let mut key_query_backrefs: Vec<Vec<(syn::Ident, syn::Ident)>> = Vec::new();
    let key_query_structs: Vec<_> = key_structs.iter().cloned()
        .map(|mut x| {
            let i = x.ident.clone();
            let mut backrefs = Vec::new();
            let mut backrefs_code = Vec::new();
            if let Some(v) = reverse_references.get(&x.ident) {
                for r in v.iter() {
                    let field = quote::format_ident!("{}_of", r.1.to_string().to_snake_case());
                    let t = &r.0;
                    backrefs.push((t.clone(), field.clone()));
                    let code = quote::quote!{
                        pub #field: KeySet<#t>,
                    };
                    println!("\ncode is {:?}", code.to_string());
                    backrefs_code.push(code);
                }
            }
            key_query_backrefs.push(backrefs);
            x.ident = quote::format_ident!("{}Query", x.ident);
            x.fields = syn::Fields::Named(syn::parse_quote!{{
                __data: #i,
                #(#backrefs_code)*
            }});
            x
        })
        .collect();
    let key_query_types: Vec<syn::PathSegment> =
        key_query_structs.iter()
        .map(|x| {
            let i = x.ident.clone();
            let g = x.generics.clone();
            syn::parse_quote!{#i#g}
        })
        .collect();

    let key_inserts: Vec<_> =
        key_structs.iter()
        .map(|x| quote::format_ident!("insert_{}", x.ident.to_string().to_snake_case()))
        .collect();
    let key_insert_backrefs: Vec<_> =
        output.key_struct_maps.iter()
        .map(|map| {
            let mut code = Vec::new();
            for (k,v) in map.iter() {
                match v {
                    KeyType::Key(t) => {
                        let field = quote::format_ident!("{}", t.to_string().to_snake_case());
                        let rev = quote::format_ident!("{}_of", k.to_string().to_snake_case());
                        code.push(quote::quote!{
                            self.#field[_datum.#k.0].#rev.insert(k);
                        });
                    }
                    KeyType::OptionKey(t) => {
                        let field = quote::format_ident!("{}", t.to_string().to_snake_case());
                        let rev = quote::format_ident!("{}_of", k.to_string().to_snake_case());
                        code.push(quote::quote!{
                            if let Some(idxk) = _datum.#k {
                                self.#field[idxk.0].#rev.insert(k);
                            }
                        });
                    }
                }
            }
            quote::quote!{
                #(#code)*
            }
        })
        .collect();
    let key_sets: Vec<_> =
        key_structs.iter()
        .map(|x| quote::format_ident!("set_{}", x.ident.to_string().to_snake_case()))
        .collect();
    let key_types: Vec<syn::PathSegment> =
        key_structs.iter()
        .map(|x| {
            let i = x.ident.clone();
            let g = x.generics.clone();
            syn::parse_quote!{#i#g}
        })
        .collect();

    // let save_enums = output.save_enums.iter();
    let table_enums = output.pod_enums.iter();
    // save_names.extend(
    //     output.save_enums.iter().map(|x| x.ident.clone()));
    let name = &input.name;
    // let savename = quote::format_ident!("{}Save", name);
    let output = quote::quote!{
        trait Query: std::ops::Deref {
            fn new(val: Self::Target) -> Self;
        }
        trait HasQuery {
            type Query: Query<Target=Self>;
        }
        #(
            #[repr(C)]
            #[derive(Eq,PartialEq,Hash,Clone)]
            #pod_structs
            #[repr(C)]
            #[derive(Eq,PartialEq,Hash,Clone)]
            /// This is plain old data.
            #pod_query_structs

            impl std::ops::Deref for #pod_query_types {
                type Target = #pod_types;
                fn deref(&self) -> &Self::Target {
                    &self.__data
                }
            }
            impl Query for #pod_query_types {
                fn new(value: Self::Target) -> Self {
                    // First pad the value with zeroes, then transmute
                    // to the query type.  This relies on zero bytes
                    // being valid values for all extra fields in the
                    // query struct.
                    #pod_query_new
                    // let x = (value,
                    //          [0u8; std::mem::size_of::<Self>() - std::mem::size_of::<Self::Target>()]);
                    // unsafe { std::mem::transmute(x) }
                }
            }
            impl HasQuery for #pod_types {
                type Query = #pod_query_types;
            }
        )*
        #(
            #[repr(C)]
            #[derive(Clone)]
            #key_structs
            #[repr(C)]
            #[derive(Clone)]
            /// This table has keys to other tables
            #key_query_structs

            impl std::ops::Deref for #key_query_types {
                type Target = #key_types;
                fn deref(&self) -> &Self::Target {
                    unsafe { &*(self as *const Self as *const Self::Target) }
                }
            }
            impl Query for #key_query_types {
                fn new(value: Self::Target) -> Self {
                    // First pad the value with zeroes, then transmute
                    // to the query type.  This relies on zero bytes
                    // being valid values for all extra fields in the
                    // query struct.
                    // unimplemented!()
                    let x = (value,
                             [0u8; std::mem::size_of::<Self>() - std::mem::size_of::<Self::Target>()]);
                    unsafe { std::mem::transmute(x) }
                }
            }
            impl HasQuery for #key_types {
                type Query = #key_query_types;
            }
        )*
        #(
            #[derive(Eq,PartialEq,Hash,Clone)]
            #table_enums
        )*

        pub struct #name {
            #(
                pub #pod_names: Vec<#pod_query_types>,
            )*
            #(
                pub #key_names: Vec<#key_query_types>,
            )*
            #(
                pub #pod_lookup_hashes: std::collections::HashMap<#pod_types, usize>,
            )*
        }
        impl #name {
            /// Create an empty #name database.
            pub fn new() -> Self {
                #name {
                    #( #pod_names: Vec::new(), )*
                    #( #key_names: Vec::new(), )*
                    #(
                        #pod_lookup_hashes: std::collections::HashMap::new(),
                    )*
                }
            }
        }

        type KeySet<T> = tinyset::Set64<Key<T>>;

        #[derive(Eq,PartialEq,Hash)]
        pub struct Key<T>(usize, std::marker::PhantomData<T>);
        impl<T> Copy for Key<T> {}
        impl<T> Clone for Key<T> {
            fn clone(&self) -> Self {
                Key(self.0, self.1)
            }
        }
        impl<T> tinyset::Fits64 for Key<T> {
            unsafe fn from_u64(x: u64) -> Self {
                Key(x as usize, std::marker::PhantomData)
            }
            fn to_u64(self) -> u64 {
                self.0.to_u64()
            }
        }

        impl #name {
            #(
                pub fn #pod_inserts(&mut self, datum: #pod_types) -> Key<#pod_types> {
                    let idx = self.#pod_names.len();
                    self.#pod_names.push(#pod_query_types::new(datum.clone()));
                    self.#pod_lookup_hashes.insert(datum, idx);
                    Key(idx, std::marker::PhantomData)
                }
            )*
            #(
                pub fn #key_inserts(&mut self, datum: #key_types) -> Key<#key_types> {
                    let idx = self.#key_names.len();
                    self.#key_names.push(#key_query_types::new(datum.clone()));
                    let k = Key(idx, std::marker::PhantomData);
                    let _datum = &self.#key_names[idx];
                    #key_insert_backrefs
                    k
                }
                pub fn #key_sets(&mut self, k: Key<#key_types>, datum: #key_types) {
                    let old = std::mem::replace(&mut self.#key_names[k.0], #key_query_types::new(datum));
                    // FIXME need to modify any back references.
                }
            )*
            #(
                pub fn #pod_lookups(&self, datum: &#pod_types) -> Option<Key<#pod_types>> {
                    self.#pod_lookup_hashes.get(datum)
                        .map(|&i| Key(i, std::marker::PhantomData))
                    // self.0.#table_names.iter().enumerate()
                    //     .filter(|&(_,x)| x == datum)
                    //     .map(|(i,x)| Key(i, std::marker::PhantomData))
                    //     .next()
                }
            )*
        }
        #(
            impl Key<#pod_types> {
                pub fn d<'a,'b>(&'a self, database: &'b #name) -> &'b #pod_query_types {
                    &database.#pod_names[self.0]
                }
            }
        )*
        #(
            impl Key<#key_types> {
                pub fn d<'a,'b>(&'a self, database: &'b #name) -> &'b #key_query_types {
                    &database.#key_names[self.0]
                }
            }
        )*
    };
    println!("\n\n\noutput is\n\n{}", output.to_string());
    output.into()
}
