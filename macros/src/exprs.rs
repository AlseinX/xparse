use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Block, Expr, ExprBlock, GenericArgument, Ident, Item, Path, PathSegment, Result, Stmt, Type,
    TypePath,
};

use self::{r#const::handle_const, r#fn::handle_fn};

mod r#const;
mod r#fn;

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
            match handle_expr(c, &mut self.current, output) {
                Ok(Some(ty_id)) => {
                    *i = GenericArgument::Type(Type::Path(TypePath {
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: Punctuated::from_iter(
                                [self.mod_id.clone(), ty_id].map(PathSegment::from),
                            ),
                        },
                    }))
                }
                Ok(None) => {}
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
    c: &mut Expr,
    current: &mut usize,
    output: &mut TokenStream,
) -> Result<Option<Ident>> {
    let ty_id = Ident::new(
        format!("Expr{current}__").as_str(),
        c.span().resolved_at(Span::mixed_site()),
    );

    'handle: {
        if let Some(i) = handle_const(c, &ty_id) {
            output.extend(quote! {
                pub(super) struct #ty_id;
                #i
            });
            break 'handle;
        }

        if let Expr::Block(ExprBlock {
            block: Block { stmts, .. },
            ..
        }) = c
        {
            if let [Stmt::Item(Item::Fn(f))] = stmts.as_mut_slice() {
                let i = handle_fn(f, &ty_id)?;

                output.extend(quote! {
                    pub(super) struct #ty_id;
                    #i
                });
                break 'handle;
            }
        }

        return Ok(None);
    }

    *current += 1;
    Ok(Some(ty_id))
}
