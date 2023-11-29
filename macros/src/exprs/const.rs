use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Block, Expr, ExprBlock, ExprCast, ExprLit, Ident, Lit, Stmt};

pub fn handle_const(expr: &Expr, ty_id: &Ident) -> Option<TokenStream> {
    if let Expr::Lit(ExprLit { lit, .. }) = expr {
        return handle_lit(lit, ty_id);
    }

    if let Expr::Block(ExprBlock {
        block: Block { stmts, .. },
        ..
    }) = expr
    {
        if let Some(Stmt::Expr(Expr::Cast(ExprCast { ty, .. }), ..)) = stmts.last() {
            return Some(const_impl(expr, ty, ty_id));
        }
    }

    None
}

fn handle_lit(lit: &Lit, ty_id: &Ident) -> Option<TokenStream> {
    Some(match lit {
        Lit::Char(lit) => const_impl(lit, &quote!(char), ty_id),
        Lit::Byte(lit) => const_impl(lit, &quote!(u8), ty_id),
        Lit::Str(lit) => const_impl(lit, &quote!(&'static str), ty_id),
        Lit::ByteStr(lit) => const_impl(lit, &quote!(&'static [u8]), ty_id),
        Lit::Int(lit) if !lit.suffix().is_empty() => {
            const_impl(lit, &Ident::new(lit.suffix(), lit.span()), ty_id)
        }
        Lit::Float(lit) => const_impl(lit, &Ident::new(lit.suffix(), lit.span()), ty_id),
        _ => return None,
    })
}

fn const_impl(expr: &impl ToTokens, ty: &impl ToTokens, ty_id: &Ident) -> TokenStream {
    quote! {
        impl ::xparse::ops::Const for #ty_id {
            type Type = #ty;
            const VALUE: Self::Type = #expr;
        }
    }
}
