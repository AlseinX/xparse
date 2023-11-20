use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Block, Error, Expr, ExprBlock, ExprCast, ExprClosure, ExprLit, ExprTuple, GenericArgument,
    Ident, Lit, Pat, PatType, Path, PathSegment, Result, ReturnType, Stmt, Type, TypePath,
};

pub fn handle_exprs(ident: &Ident, input: &mut Type) -> Result<TokenStream> {
    let mod_id = Ident::new(
        format!("__{ident}_exprs").as_str(),
        ident.span().resolved_at(Span::mixed_site()),
    );
    let mut visitor = ExprVisitor {
        mod_id: mod_id.clone(),
        output: Ok(TokenStream::new()),
        current: 0,
    };
    visitor.visit_type_mut(input);
    visitor.output.map(|o| {
        if o.is_empty() {
            o
        } else {
            quote! {
                #[allow(non_snake_case)]
                #[doc(hidden)]
                mod #mod_id {
                    use super::*;
                    #o
                }
            }
        }
    })
}

struct ExprVisitor {
    mod_id: Ident,
    output: Result<TokenStream>,
    current: usize,
}

impl VisitMut for ExprVisitor {
    fn visit_generic_argument_mut(&mut self, i: &mut GenericArgument) {
        let Ok(output) = &mut self.output else {
            return;
        };
        if let GenericArgument::Const(c) = i {
            let c = std::mem::replace(
                c,
                Expr::Tuple(ExprTuple {
                    attrs: Default::default(),
                    paren_token: Default::default(),
                    elems: Default::default(),
                }),
            );
            match handle_expr(c, &self.mod_id, &mut self.current, output) {
                Ok(arg) => *i = arg,
                Err(e) => {
                    self.output = Err(e);
                    return;
                }
            }
        }
        visit_mut::visit_generic_argument_mut(self, i)
    }
}

fn handle_expr(
    expr: Expr,
    mod_id: &Ident,
    current: &mut usize,
    output: &mut TokenStream,
) -> Result<GenericArgument> {
    let ty_id = Ident::new(
        format!("Expr{current}__").as_str(),
        expr.span().resolved_at(Span::mixed_site()),
    );
    *current += 1;

    let span = expr.span();

    'def: {
        match expr {
            Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }) => {
                output.extend(quote! {
                    pub(super) struct #ty_id;
                    impl ::xparse::ops::Const for #ty_id {
                        type Type = &'static str;
                        const VALUE: Self::Type = #s;
                    }
                });
                break 'def;
            }
            Expr::Lit(ExprLit {
                lit: Lit::ByteStr(s),
                ..
            }) => {
                output.extend(quote! {
                    pub(super) struct #ty_id;
                    impl ::xparse::ops::Const for #ty_id {
                        type Type = &'static [u8];
                        const VALUE: Self::Type = #s;
                    }
                });
                break 'def;
            }
            Expr::Lit(ExprLit {
                lit: Lit::Char(c), ..
            }) => {
                output.extend(quote! {
                    pub(super) struct #ty_id;
                    impl ::xparse::ops::Const for #ty_id {
                        type Type = char;
                        const VALUE: Self::Type = #c;
                    }
                });
                break 'def;
            }
            Expr::Lit(ExprLit {
                lit: Lit::Byte(c), ..
            }) => {
                output.extend(quote! {
                    pub(super) struct #ty_id;
                    impl ::xparse::ops::Const for #ty_id {
                        type Type = u8;
                        const VALUE: Self::Type = #c;
                    }
                });
                break 'def;
            }
            _ => (),
        }

        let Expr::Block(ExprBlock {
            block: Block { stmts, .. },
            ..
        }) = expr
        else {
            return Ok(GenericArgument::Const(expr));
        };

        let stmts = match <[_; 1]>::try_from(stmts) {
            Ok(
                [Stmt::Expr(
                    Expr::Closure(ExprClosure {
                        attrs,
                        lifetimes,
                        constness,
                        movability,
                        asyncness,
                        capture,
                        inputs,
                        output: rtn_ty,
                        body,
                        ..
                    }),
                    None,
                )],
            ) => {
                let mut pt = Vec::with_capacity(inputs.len());
                let mut pv = Vec::with_capacity(inputs.len());
                for input in inputs {
                    let Pat::Type(PatType { attrs, pat, ty, .. }) = input else {
                        return Err(Error::new(input.span(), "need explicit type annotation"));
                    };

                    if let Some(f) = attrs.first() {
                        return Err(Error::new(f.span(), "attributes are not supported here"));
                    }

                    pt.push(ty);
                    pv.push(pat);
                }

                let ReturnType::Type(.., rtn_ty) = rtn_ty else {
                    return Err(Error::new(rtn_ty.span(), "need explicit return type"));
                };

                let lifetimes = lifetimes.map(|x| x.lifetimes).into_iter();

                let (None, None, None, None) = (constness, movability, asyncness, capture) else {
                    return Err(Error::new(constness.span(), "modifiers not supported here"));
                };

                let body = if let Expr::Block(ExprBlock {
                    block: Block { stmts, .. },
                    ..
                }) = body.as_ref()
                {
                    quote!(#(#stmts)*)
                } else {
                    quote!(body)
                };

                output.extend(quote! {
                    pub(super) struct #ty_id;
                    impl #(<#lifetimes>)* ::xparse::ops::Mapper<(#(#pt,)*)> for #ty_id {
                        type Output = #rtn_ty;
                        #(#attrs)*
                        #[inline]
                        fn map((#(#pv,)*): (#(#pt,)*)) -> Self::Output {
                            #body
                        }
                    }
                });
                break 'def;
            }
            Ok([stmt]) => vec![stmt],
            Err(stmts) => stmts,
        };

        let Some(Stmt::Expr(Expr::Cast(ExprCast { ty: rtn_ty, .. }), None)) = stmts.last() else {
            return Err(Error::new(
                span,
                "must return with `as` explicit type annotation",
            ));
        };

        let value = if let [Stmt::Expr(expr, None)] = stmts.as_slice() {
            quote!(#expr)
        } else {
            quote!({#(#stmts)*})
        };

        output.extend(quote! {
            pub(super) struct #ty_id;
            impl ::xparse::ops::Const for #ty_id {
                type Type = #rtn_ty;
                const VALUE: Self::Type = #value;
            }
        });
    }

    Ok(GenericArgument::Type(Type::Path(TypePath {
        qself: None,
        path: Path {
            leading_colon: None,
            segments: Punctuated::from_iter([mod_id.clone(), ty_id].map(PathSegment::from)),
        },
    })))
}
