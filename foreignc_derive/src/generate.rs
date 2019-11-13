use super::*;

use super::error::*;

use core::convert::From;
use quote::*;
use std::iter::{Extend, FromIterator};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::*;

pub fn to_extern_item_fn(
    mut item: ItemFn,
    implm: Option<(&Type, Ident)>,
) -> DResult<ItemFn> {
    let mut itemc = item.clone();
    let identc = Ident::new(&format!("{}_ffi", itemc.sig.ident), item.sig.ident.span());
    itemc.sig.ident = identc.clone();

    let mut args: Vec<&Pat> = Vec::new();

    for ty in &mut item.sig.inputs {
        match ty {
            // Convert self type
            FnArg::Receiver(_) => {
                return Err(syn::Error::new(Span::call_site(), "Cannot have self in item fn").into());
            }
            // Use arg type
            FnArg::Typed(ref mut t) => {
                // [some_name]: [&|&mut] [some_type]
                // Convert arguments from [&|&mut] to [*const|*mut]
                if let Pat::Ident(ref mut p) = &mut *t.pat {
                    p.mutability = None;
                }
                args.push(&*t.pat);
                let ty = convert_to_ptr(&t.ty)?;
                t.ty = ty;
            }
        }
    }

    let old_output = if let ReturnType::Type(_, ty) = item.sig.output {
        ty.clone()
    }else {
        Box::new(parse_str("()").unwrap())
    };

    item.sig.output = ReturnType::Type(Token![->](itemc.span().clone()), Box::new(parse_str("foreignc::CArgResult").unwrap()));

    Ok(ItemFn {
        block: Box::new(
            parse(if let Some((caller, method_name)) = implm {
                quote!(
                    {
                        unsafe {
                            || -> foreignc::ArgResult<_> {
                                Ok(
                                    #caller::#method_name(#(
                                        foreignc::FromFFi::from_ffi(#args)?
                                    ),*)
                                )
                            }().into()
                        }
                    }
                )
                .into()
            } else {
                quote!(
                    {
                        #itemc
                        unsafe {
                            || -> foreignc::ArgResult<_> {
                                Ok(
                                    #identc(#(
                                        foreignc::FromFFi::from_ffi(#args)?
                                    ),*)
                                )
                            }().into()
                        }
                    }
                )
                .into()
            })?
            ,
        ),
        vis: VisPublic {
            pub_token: Token![pub](item.sig.span()),
        }
        .into(),
        attrs: Vec::new(),
        sig: Signature {
            abi: Some(Abi {
                extern_token: Token![extern](item.sig.span()),
                name: Some(LitStr::new("C", item.sig.span())),
            }),
            ..item.sig
        },
    })
}

pub fn convert_item_fn(self_ty: &Box<Type>, item_fn: ImplItemMethod) -> DResult<ItemFn> {
    let mut inputs = Vec::new();
    for i in &item_fn.sig.inputs {
        let p_ty = if let FnArg::Receiver(r) = i {
            PatType {
                attrs: Vec::new(),
                pat: Box::new(
                    PatIdent {
                        attrs: Vec::new(),
                        by_ref: None,
                        mutability: r.mutability,
                        ident: Ident::new("this", r.span()),
                        subpat: None,
                    }
                    .into(),
                ),
                colon_token: Token![:](r.span()),
                ty: {
                    if r.reference.is_some() {
                        Box::new(
                            TypeReference {
                                and_token: Token![&](r.span()),
                                lifetime: None,
                                mutability: r.mutability,
                                elem: self_ty.clone(),
                            }
                            .into(),
                        )
                    } else {
                        self_ty.clone()
                    }
                },
            }
            .into()
        } else {
            i.clone()
        };
        inputs.push(p_ty);
    }
    Ok(ItemFn {
        vis: item_fn.vis,
        attrs: item_fn.attrs,
        sig: Signature {
            inputs: Punctuated::from_iter(inputs.into_iter()),
            ident: Ident::new(
                &to_snake_case(format!(
                    "{}{}",
                    &item_fn.sig.ident,
                    if let Type::Path(ref p) = &*self_ty.clone() {
                        p.path.segments[0].ident.to_string()
                    } else {
                        return Err(syn::Error::new(Span::call_site(), "Failed to get self type name").into());
                    }
                )),
                item_fn.sig.ident.span(),
            ),
            ..item_fn.sig
        },
        block: Box::new(item_fn.block),
    })
}

pub fn convert_to_ptr(ty: &Box<Type>) -> DResult<Box<Type>> {
    match &**ty {
        Type::Reference(ref r) => convert_to_ptr(&r.elem),
        Type::Path(ref path) => {
            let seg0 = &path.path.segments[0];
            let path_name = seg0.ident.to_string();
            if path_name == "Result" || path_name == "Option" {
                if let PathArguments::AngleBracketed(ref inner) = seg0.arguments {
                    if let GenericArgument::Type(ref inner_ty) = inner.args[0] {
                        let t = Box::new(inner_ty.clone());
                        Ok(Box::new(TypePtr {
                            star_token: Token![*](ty.span()),
                            const_token: None,
                            mutability: Some(Token![mut](ty.span())),
                            elem: Box::new(parse_str(&format!("C{}", path_name)).unwrap()),
                        }.into()))
                    } else {
                        return Err(syn::Error::new(Span::call_site(), "Result or option should not have lifetime").into());
                    }
                } else {
                    return Err(syn::Error::new(Span::call_site(), "Expected generic arguments after Result or Option").into());
                }
            } else {
                match path_name.as_str() {
                    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16"
                    | "u32" | "u64" | "u128" | "usize" | "f32" | "f64" | "bool"
                    | "char" => Ok(ty.clone()),
                    _ => Ok(Box::new(
                        TypePtr {
                            star_token: Token![*](ty.span()),
                            const_token: None,
                            mutability: Some(Token![mut](ty.span())),
                            elem: Box::new(parse_str("std::ffi::c_void").unwrap()),
                        }
                        .into()),
                    ),
                }
            }
        }
        Type::Ptr(_) => Ok(ty.clone()),
        _ => Ok(Box::new(
            TypePtr {
                star_token: Token![*](ty.span()),
                const_token: None,
                mutability: Some(Token![mut](ty.span())),
                elem: ty.clone(),
            }
            .into()),
        ),
    }
}
