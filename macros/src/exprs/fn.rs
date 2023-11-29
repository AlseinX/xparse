use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    spanned::Spanned, token::Paren, Error, FnArg, Generics, Ident, ItemFn, Pat, PatIdent, PatType,
    Result, ReturnType, Signature, Token, Type, TypeParam, TypePath, TypeReference, TypeTuple,
    Visibility,
};

pub fn handle_fn(f: &mut ItemFn, ty_id: &Ident) -> Result<TokenStream> {
    let (Visibility::Inherited, None, None, None, None) = (
        &f.vis,
        &f.sig.constness,
        &f.sig.asyncness,
        &f.sig.abi,
        &f.sig.variadic,
    ) else {
        return Err(Error::new(
            f.span(),
            "visibility, const, async, abi, or variadic params are not supported here",
        ));
    };

    match f.sig.ident.to_string().as_str() {
        "map" => handle_map(f, ty_id),
        "is" => handle_is(f, ty_id),
        other => Err(Error::new(
            f.sig.ident.span(),
            format_args!("unexpected function {other}"),
        )),
    }
}

fn handle_map(
    ItemFn {
        attrs,
        sig:
            Signature {
                unsafety,
                fn_token,
                generics:
                    Generics {
                        lt_token,
                        params,
                        gt_token,
                        where_clause,
                    },
                inputs,
                output: rtn_ty,
                ..
            },
        block,
        ..
    }: &ItemFn,
    ty_id: &Ident,
) -> Result<TokenStream> {
    let mut pt = Vec::with_capacity(inputs.len());
    let mut pv = Vec::with_capacity(inputs.len());

    for input in inputs {
        let FnArg::Typed(PatType { attrs, pat, ty, .. }) = input else {
            return Err(Error::new(
                input.span(),
                "self receievers are not supposed to be here",
            ));
        };

        if let Some(f) = attrs.first() {
            return Err(Error::new(f.span(), "attributes are not supported here"));
        }

        pt.push(ty);
        pv.push(pat);
    }

    let rtn_ty = if let ReturnType::Type(_, rtn_ty) = rtn_ty {
        rtn_ty.as_ref().clone()
    } else {
        Type::Tuple(TypeTuple {
            paren_token: Paren(rtn_ty.span()),
            elems: Default::default(),
        })
    };

    Ok(quote! {
        impl #lt_token #params #gt_token ::xparse::ops::Mapper<(#(#pt,)*)> for #ty_id #where_clause {
            type Output = #rtn_ty;
            #(#attrs)*
            #[inline]
            #unsafety #fn_token map((#(#pv,)*): (#(#pt,)*)) -> Self::Output #block
        }
    })
}

fn handle_is(f: &mut ItemFn, ty_id: &Ident) -> Result<TokenStream> {
    let span = f.sig.span().resolved_at(Span::mixed_site());
    let Signature {
        generics, inputs, ..
    } = &mut f.sig;

    if inputs.is_empty() {
        return Err(Error::new(
            inputs.span(),
            "a parameter as the input is required",
        ));
    }

    let a = if let Some(FnArg::Typed(PatType { ty: i, .. })) = inputs.iter().nth(1) {
        i
    } else {
        let ident = Ident::new("__Arg", span);
        generics.lt_token = Some(Token![<](span));
        generics.params.push(syn::GenericParam::Type(TypeParam {
            attrs: Default::default(),
            ident: ident.clone(),
            colon_token: None,
            bounds: Default::default(),
            eq_token: None,
            default: None,
        }));
        generics.gt_token = Some(Token![>](span));
        inputs.push(FnArg::Typed(PatType {
            attrs: Default::default(),
            pat: Box::new(Pat::Ident(PatIdent {
                attrs: Default::default(),
                by_ref: None,
                mutability: None,
                ident: Ident::new("_", span),
                subpat: None,
            })),
            colon_token: Token![:](span),
            ty: Box::new(Type::Path(TypePath {
                qself: None,
                path: ident.into(),
            })),
        }));
        let Some(FnArg::Typed(PatType { ty, .. })) = inputs.iter().nth(1) else {
            unreachable!()
        };
        ty
    };

    let Generics {
        lt_token,
        params,
        gt_token,
        where_clause,
    } = std::mem::replace(
        generics,
        Generics {
            lt_token: None,
            params: Default::default(),
            gt_token: None,
            where_clause: None,
        },
    );

    let Some(FnArg::Typed(PatType { ty: i, .. })) = inputs.first() else {
        return Err(Error::new(
            inputs.span(),
            "a parameter as the input is required",
        ));
    };

    let Type::Reference(TypeReference { elem: i, .. }) = i.as_ref() else {
        return Err(Error::new(
            inputs.span(),
            "the input parameter has to be an reference",
        ));
    };

    Ok(quote! {
        impl #lt_token #params #gt_token ::xparse::ops::Predicate<#i, #a> for #ty_id #where_clause {
            #f
        }
    })
}
