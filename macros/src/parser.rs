use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    token::Paren,
    AngleBracketedGenericArguments, AssocType, ConstParam, Error, Expr, ExprLit, ExprPath,
    GenericArgument, GenericParam, Generics, ItemType, LifetimeParam, Lit, LitStr, MacroDelimiter,
    Meta, MetaList, Path, PathArguments, PathSegment, PredicateType, Result, Token, TraitBound,
    TraitBoundModifier, Type, TypeParam, TypeParamBound, TypePath, TypeTuple, WherePredicate,
};

use crate::exprs::handle_exprs;

pub struct Args {
    rec: Option<(Type, Type)>,
    arg: Option<Type>,
}

impl Parse for Args {
    fn parse(i: ParseStream) -> Result<Self> {
        if i.is_empty() {
            return Ok(Self {
                rec: None,
                arg: None,
            });
        }
        let span = i.span();
        let g = i.parse_terminated(GenericArgument::parse, Token![,])?;
        let mut input = None;
        let mut output = None;
        let mut arg = None;
        'args: for a in g {
            match a {
                GenericArgument::Type(t) => {
                    for slot in [&mut input, &mut output, &mut arg] {
                        if slot.is_some() {
                            continue;
                        }
                        *slot = Some(t);
                        continue 'args;
                    }
                    return Err(Error::new(t.span(), "too many arguments"));
                }
                GenericArgument::AssocType(AssocType {
                    ident,
                    generics: None,
                    ty,
                    ..
                }) => {
                    *match ident.to_string().as_str() {
                        "input" => &mut input,
                        "output" => &mut output,
                        "arg" => &mut arg,
                        _ => return Err(Error::new(span, "unexpected argument")),
                    } = Some(ty)
                }
                _ => return Err(Error::new(span, "unexpected argument")),
            }
        }
        let rec = match (input, output) {
            (None, None) => None,
            (Some(input), Some(output)) => Some((input, output)),
            _ => return Err(Error::new(span, "must specify both input and output")),
        };
        Ok(Self { rec, arg })
    }
}

pub fn handle(
    ItemType {
        mut attrs,
        vis,
        ident,
        mut generics,
        ty,
        semi_token,
        ..
    }: ItemType,
    input_args: Args,
) -> Result<TokenStream> {
    let mut ty = if let Some((name, i)) = attrs.iter().enumerate().find_map(|(i, attr)| match &attr
        .meta
    {
        Meta::Path(path) if path.is_ident("name") => {
            Some((LitStr::new(ident.to_string().as_str(), path.span()), i))
        }
        Meta::List(MetaList {
            path,
            delimiter: MacroDelimiter::Paren(..),
            tokens,
        }) if path.is_ident("name") => parse2(tokens.clone())
            .map(|x: Ident| LitStr::new(x.to_string().as_str(), x.span()))
            .or_else(|_| parse2(tokens.clone()))
            .ok()
            .map(|name| (name, i)),
        _ => None,
    }) {
        attrs.remove(i);
        let span = ty.span().resolved_at(Span::mixed_site());
        Type::Path(TypePath {
            qself: None,
            path: Path {
                leading_colon: Some(Token![::](span)),
                segments: Punctuated::from_iter(
                    ["xparse", "ops"]
                        .iter()
                        .map(|x| PathSegment {
                            ident: Ident::new(x, span),
                            arguments: PathArguments::None,
                        })
                        .chain([PathSegment {
                            ident: Ident::new("Name", span),
                            arguments: PathArguments::AngleBracketed(
                                AngleBracketedGenericArguments {
                                    colon2_token: None,
                                    lt_token: Token![<](span),
                                    args: Punctuated::from_iter([
                                        GenericArgument::Type(*ty),
                                        GenericArgument::Const(Expr::Lit(ExprLit {
                                            attrs: Default::default(),
                                            lit: Lit::Str(name),
                                        })),
                                    ]),
                                    gt_token: Token![>](span),
                                },
                            ),
                        }]),
                ),
            },
        })
    } else {
        *ty
    };

    let expr_defs = handle_exprs(&ident, &mut ty)?;
    let span = generics.span().resolved_at(Span::mixed_site());
    let struct_generics = generics.clone();
    let args = generic_params_to_args(&generics.params);
    let (args_lt, args_rt) = if args.is_empty() {
        (Some(Token![<](span)), Some(Token![>](span)))
    } else {
        (None, None)
    };

    let arg = if let Some(arg) = input_args.arg {
        arg
    } else {
        let arg = Ident::new("__Arg", span);
        let at = ident_to_type(arg.clone());
        generics.lt_token.get_or_insert_with(|| Token![<](span));
        generics.gt_token.get_or_insert_with(|| Token![>](span));
        generics.params.push(GenericParam::Type(TypeParam {
            attrs: Default::default(),
            ident: arg,
            colon_token: Some(Token![:](span)),
            bounds: Default::default(),
            eq_token: Default::default(),
            default: Default::default(),
        }));
        at
    };

    let (input, output) = if let Some((input, output)) = input_args.rec {
        (
            input,
            Type::Tuple(TypeTuple {
                paren_token: Paren(span),
                elems: Punctuated::from_iter([output]),
            }),
        )
    } else {
        let input = Ident::new("__Input", span);
        let output = Ident::new("__Output", span);
        let it = ident_to_type(input.clone());
        let ot = ident_to_type(output.clone());
        generics.lt_token.get_or_insert_with(|| Token![<](span));
        generics.gt_token.get_or_insert_with(|| Token![>](span));
        generics.params.push(ident_to_type_param(input));
        generics.params.push(ident_to_type_param(output));
        generics
            .make_where_clause()
            .predicates
            .push(where_ty_impls_parse_impl(&ty, &it, &ot, &arg, span));
        (it, ot)
    };

    let Generics {
        lt_token,
        params,
        gt_token,
        where_clause,
    } = &generics;

    let s = Ident::new("__Source", Span::mixed_site());
    let i = Ident::new("__Input", Span::mixed_site());
    let a = Ident::new("__Arg", Span::mixed_site());

    let f = quote! {
        #[inline(always)]
        fn parse<#s: ::xparse::Source<Item = #input>>(input: &mut #s, arg: &#arg) -> Result<Self::Output> {
            <#ty as ::xparse::parse::ParseImpl<#input, #arg>>::parse(input, arg)
        }
    };

    #[cfg(feature = "async")]
    let f = quote! {
        #f
        #[inline(always)]
        async fn parse_async<S: ::xparse::AsyncSource<Item = #input>>(input: &mut S, arg: &#arg) -> Result<Self::Output> {
            Box::pin(<#ty as ::xparse::parse::ParseImpl<#input, #arg>>::parse_async(input, arg)).await
        }
    };

    Ok(quote! {
        #expr_defs
        #(#attrs)* #vis struct #ident #struct_generics #semi_token
        impl #lt_token #params #gt_token ::xparse::parse::ParseImpl<#input, #arg> for #ident #args_lt #args #args_rt #where_clause {
            type Output = #output;
            #f
        }

        impl<#i, #a> ::xparse::ops::Predicate<#i, #a> for #ident
        where
            #ty: ::xparse::ops::Predicate<#i, #a>
        {
            #[inline(always)]
            fn is(v: &#i, arg: &#a) -> bool {
                <#ty as ::xparse::ops::Predicate<#i, #a>>::is(v, arg)
            }
        }

        impl<#i, #a> ::xparse::ops::Mapper<#i, #a> for #ident
        where
            #ty: ::xparse::ops::Mapper<#i, #a>
        {
            type Output = <#ty as ::xparse::ops::Mapper<#i, #a>>::Output;
            #[inline(always)]
            fn map(v: #i, a: &#a) -> Self::Output {
                <#ty as ::xparse::ops::Mapper<#i, #a>>::map(v, a)
            }
        }
    })
}

fn where_ty_impls_parse_impl(
    ty: &Type,
    it: &Type,
    ot: &Type,
    at: &Type,
    span: Span,
) -> WherePredicate {
    WherePredicate::Type(PredicateType {
        lifetimes: None,
        bounded_ty: ty.clone(),
        colon_token: Token![:](span),
        bounds: Punctuated::from_iter([TypeParamBound::Trait(TraitBound {
            paren_token: None,
            modifier: TraitBoundModifier::None,
            lifetimes: None,
            path: Path {
                leading_colon: Some(Token![::](span)),
                segments: Punctuated::from_iter(
                    ["xparse", "parse"]
                        .iter()
                        .map(|x| PathSegment {
                            ident: Ident::new(x, span),
                            arguments: PathArguments::None,
                        })
                        .chain([PathSegment {
                            ident: Ident::new("ParseImpl", span),
                            arguments: PathArguments::AngleBracketed(
                                AngleBracketedGenericArguments {
                                    colon2_token: None,
                                    lt_token: Token![<](span),
                                    args: Punctuated::from_iter([
                                        GenericArgument::Type(it.clone()),
                                        GenericArgument::Type(at.clone()),
                                        GenericArgument::AssocType(AssocType {
                                            ident: Ident::new("Output", span),
                                            generics: None,
                                            eq_token: Token![=](span),
                                            ty: ot.clone(),
                                        }),
                                    ]),
                                    gt_token: Token![>](span),
                                },
                            ),
                        }]),
                ),
            },
        })]),
    })
}

fn generic_params_to_args(
    params: &Punctuated<GenericParam, Token![,]>,
) -> Punctuated<GenericArgument, Token![,]> {
    params
        .iter()
        .map(|p| match p {
            GenericParam::Lifetime(LifetimeParam { lifetime, .. }) => {
                GenericArgument::Lifetime(lifetime.clone())
            }
            GenericParam::Type(TypeParam { ident, .. }) => {
                GenericArgument::Type(Type::Path(TypePath {
                    qself: None,
                    path: ident.clone().into(),
                }))
            }
            GenericParam::Const(ConstParam { attrs, ident, .. }) => {
                GenericArgument::Const(Expr::Path(ExprPath {
                    attrs: attrs.clone(),
                    qself: None,
                    path: ident.clone().into(),
                }))
            }
        })
        .collect()
}

fn ident_to_type_param(ident: Ident) -> GenericParam {
    GenericParam::Type(TypeParam {
        attrs: Default::default(),
        ident,
        colon_token: None,
        bounds: Default::default(),
        eq_token: Default::default(),
        default: Default::default(),
    })
}

fn ident_to_type(ident: Ident) -> Type {
    Type::Path(TypePath {
        qself: None,
        path: ident.into(),
    })
}
