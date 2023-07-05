use syn::{parse::Parse, BinOp, Expr};

use crate::syn_macros::*;

ast_enum_of_structs! {
    pub enum CoreExpr {
        Assignment(Assignment),
        StateBind(StateBind),
        FnBind(FnBind),
        Closure(Closure),
        Expr(Expr),
    }
}

impl Parse for CoreExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let left = if input.peek(syn::Token![$]) {
            CoreExpr::StateBind(input.parse()?)
        } else if input.peek(syn::Token![@]) {
            CoreExpr::FnBind(input.parse()?)
        } else if input.peek(syn::token::Brace) {
            CoreExpr::Closure(input.parse()?)
        } else {
            CoreExpr::Expr(input.parse()?)
        };

        // let expr = input.parse()?;
        if input.peek(syn::Token![+=])
            || input.peek(syn::Token![-=])
            || input.peek(syn::Token![*=])
            || input.peek(syn::Token![/=])
            || input.peek(syn::Token![%=])
            || input.peek(syn::Token![<<=])
            || input.peek(syn::Token![>>=])
            || input.peek(syn::Token![|=])
            || input.peek(syn::Token![&=])
            || input.peek(syn::Token![^=])
        {
            let op = input.parse()?;
            let right = input.parse()?;

            return Ok(CoreExpr::Assignment(Assignment {
                left: Box::new(left),
                op,
                right,
            }));
        }
        Ok(left)
    }
}

fn is_op_assign(op: &BinOp) -> bool {
    match op {
        BinOp::AddAssign(_)
        | BinOp::SubAssign(_)
        | BinOp::MulAssign(_)
        | BinOp::DivAssign(_)
        | BinOp::RemAssign(_)
        | BinOp::ShlAssign(_)
        | BinOp::ShrAssign(_)
        | BinOp::BitOrAssign(_)
        | BinOp::BitAndAssign(_)
        | BinOp::BitXorAssign(_) => true,
        _ => false,
    }
}

ast_struct! {
    pub struct Assignment #full {
        pub left: Box<CoreExpr>,
        pub op: BinOp,
        pub right: Box<CoreExpr>,
    }
}

impl syn::parse::Parse for Assignment {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let left = input.parse()?;
        let op = input.parse()?;
        let right = input.parse()?;

        Ok(Assignment { left, op, right })
    }
}

ast_struct! {
    pub struct Closure #full {
        pub brace: syn::token::Brace,
        pub expr: Box<CoreExpr>,
    }
}

impl Parse for Closure {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Closure {
            brace: syn::braced!(content in input),
            expr: content.parse()?,
        })
    }
}

ast_struct! {
    pub struct StateBind #full {
        pub bind_token: syn::Token![$],
        pub ident: syn::Ident,
    }
}

impl syn::parse::Parse for StateBind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(StateBind {
            bind_token: input.parse()?,
            ident: input.parse()?,
        })
    }
}

ast_struct! {
    pub struct FnBind #full {
        pub bind_token: syn::Token![@],
        pub ident: syn::Ident,
    }
}

impl syn::parse::Parse for FnBind {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(FnBind {
            bind_token: input.parse()?,
            ident: input.parse()?,
        })
    }
}
