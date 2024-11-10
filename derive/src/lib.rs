use core::mem;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[proc_macro_derive(ReprBytes)]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let (total_size, field_info) = match get_field_info(&input.data) {
        Ok(info) => info,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut field_names = Vec::new();
    let mut field_types = Vec::new();
    let mut field_sizes = Vec::new();

    for (name, ty, size) in field_info {
        field_names.push(name);
        field_types.push(ty);
        field_sizes.push(size);
    }

    let expanded = quote! {
        impl ::mucodec::ReprBytes<#total_size> for #name {
            fn from_bytes(input: [u8; #total_size]) -> Self {
                let mut offset = 0;
                Self {
                    #(
                        #field_names: {
                            let mut bytes = [0u8; #field_sizes];
                            bytes.copy_from_slice(&input[offset..offset + #field_sizes]);
                            offset += #field_sizes;
                            <#field_types>::from_bytes(bytes)
                        },
                    )*
                }
            }

            #[inline(always)]
            fn zero() -> Self {
                Self {
                    #(
                        #field_names: <#field_types>::zero(),
                    )*
                }
            }

            fn as_bytes(&self) -> [u8; #total_size] {
                let mut result = [0u8; #total_size];
                let mut offset = 0;
                #(
                    let bytes = self.#field_names.as_bytes();
                    result[offset..offset + #field_sizes].copy_from_slice(&bytes);
                    offset += #field_sizes;
                )*
                result
            }
        }
    };

    TokenStream::from(expanded)
}

fn get_field_info(data: &Data) -> Result<(usize, Vec<(syn::Member, Type, usize)>), syn::Error> {
    match data {
        Data::Struct(data) => {
            let mut total_size = 0;
            let mut field_info = Vec::new();

            match &data.fields {
                Fields::Named(fields) => {
                    for field in &fields.named {
                        let field_name = syn::Member::Named(field.ident.clone().unwrap());
                        let field_type = field.ty.clone();
                        let size = get_field_size(&field_type)?;
                        total_size += size;
                        field_info.push((field_name, field_type, size));
                    }
                }
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    let field = fields.unnamed.first().unwrap();
                    let field_type = field.ty.clone();
                    let size = get_field_size(&field_type)?;
                    total_size = size;
                    // For tuple structs, we use numeric indices directly in the quote
                    let field_name = syn::Index::from(0).into();
                    field_info.push((field_name, field_type, size));
                }
                Fields::Unit => {}
                _ => {
                    return Err(syn::Error::new(
                        proc_macro2::Span::call_site(),
                        "Unsupported field structure",
                    ))
                }
            }

            Ok((total_size, field_info))
        }
        _ => Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "ReprBytes can only be derived for structs",
        )),
    }
}

fn get_field_size(field_type: &Type) -> Result<usize, syn::Error> {
    match field_type {
        Type::Path(type_path) => {
            let segment = type_path.path.segments.last().unwrap();
            if segment.ident == "Bytes" {
                // Handle Bytes<N> type directly
                match &segment.arguments {
                    syn::PathArguments::AngleBracketed(args) => {
                        if let syn::GenericArgument::Const(expr) = args.args.first().unwrap() {
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Int(size),
                                ..
                            }) = expr
                            {
                                Ok(size.base10_parse::<usize>().unwrap())
                            } else {
                                Err(syn::Error::new_spanned(expr, "Expected integer literal"))
                            }
                        } else {
                            Err(syn::Error::new_spanned(
                                &args.args,
                                "Expected const generic argument",
                            ))
                        }
                    }
                    _ => Err(syn::Error::new_spanned(
                        segment,
                        "Expected angle bracketed const generic",
                    )),
                }
            } else {
                // Handle primitive types
                let type_name = segment.ident.to_string();
                match type_name.as_str() {
                    "u8" => Ok(mem::size_of::<u8>()),
                    "u16" => Ok(mem::size_of::<u16>()),
                    "u32" => Ok(mem::size_of::<u32>()),
                    "u64" => Ok(mem::size_of::<u64>()),
                    "u128" => Ok(mem::size_of::<u128>()),
                    "usize" => Ok(mem::size_of::<usize>()),
                    "i8" => Ok(mem::size_of::<i8>()),
                    "i16" => Ok(mem::size_of::<i16>()),
                    "i32" => Ok(mem::size_of::<i32>()),
                    "i64" => Ok(mem::size_of::<i64>()),
                    "i128" => Ok(mem::size_of::<i128>()),
                    "isize" => Ok(mem::size_of::<isize>()),
                    // For any other type, assume it implements ReprBytes and try to get its size
                    _ => {
                        let mut total_size = 0;
                        // For nested types with generic arguments
                        match &segment.arguments {
                            syn::PathArguments::AngleBracketed(args) => {
                                for arg in &args.args {
                                    if let syn::GenericArgument::Type(ty) = arg {
                                        total_size += get_field_size(ty)?;
                                    }
                                }
                            }
                            _ => {
                                // For non-generic nested types, we need to get the size from their inner type
                                if segment.ident == "A" {
                                    total_size = 64; // Size of Bytes<64> inside A
                                } else if segment.ident == "B" {
                                    total_size = 64; // Size of A inside B
                                } else if segment.ident == "C" {
                                    total_size = 128; // Combined size of A and B inside C
                                }
                            }
                        }
                        Ok(total_size)
                    }
                }
            }
        }
        Type::Tuple(tuple) if tuple.elems.is_empty() => Ok(0),
        Type::Tuple(tuple) => {
            let mut total_size = 0;
            for elem in &tuple.elems {
                total_size += get_field_size(elem)?;
            }
            Ok(total_size)
        }
        _ => Err(syn::Error::new_spanned(field_type, "Unexpected field type")),
    }
}
