use syn::{parse::Parse, Expr};

use crate::syn_macros::*;

ast_enum! {
    pub enum BinOp {
        /// The `=` operator
        Eq(syn::Token![=]),
        /// The `+=` operator
        AddEq(syn::Token![+=]),
        /// The `-=` operator
        SubEq(syn::Token![-=]),
        /// The `*=` operator
        MulEq(syn::Token![*=]),
        /// The `/=` operator
        DivEq(syn::Token![/=]),
        /// The `%=` operator
        RemEq(syn::Token![%=]),
        /// The `^=` operator
        BitXorEq(syn::Token![^=]),
        /// The `&=` operator
        BitAndEq(syn::Token![&=]),
        /// The `|=` operator
        BitOrEq(syn::Token![|=]),
        /// The `<<=` operator
        ShlEq(syn::Token![<<=]),
        /// The `>>=` operator
        ShrEq(syn::Token![>>=]),
    }
}

impl Parse for BinOp {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::Token![=]) {
            input.parse().map(BinOp::Eq)
        } else if input.peek(syn::Token![+=]) {
            input.parse().map(BinOp::AddEq)
        } else if input.peek(syn::Token![-=]) {
            input.parse().map(BinOp::SubEq)
        } else if input.peek(syn::Token![*=]) {
            input.parse().map(BinOp::MulEq)
        } else if input.peek(syn::Token![/=]) {
            input.parse().map(BinOp::DivEq)
        } else if input.peek(syn::Token![%=]) {
            input.parse().map(BinOp::RemEq)
        } else if input.peek(syn::Token![^=]) {
            input.parse().map(BinOp::BitXorEq)
        } else if input.peek(syn::Token![&=]) {
            input.parse().map(BinOp::BitAndEq)
        } else if input.peek(syn::Token![|=]) {
            input.parse().map(BinOp::BitOrEq)
        } else if input.peek(syn::Token![<<=]) {
            input.parse().map(BinOp::ShlEq)
        } else if input.peek(syn::Token![>>=]) {
            input.parse().map(BinOp::ShrEq)
        } else {
            Err(input.error("expected binary operator"))
        }
    }
}

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
        if input.peek(syn::Token![=])
            || input.peek(syn::Token![+=])
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
        BinOp::AddEq(_)
        | BinOp::SubEq(_)
        | BinOp::MulEq(_)
        | BinOp::DivEq(_)
        | BinOp::RemEq(_)
        | BinOp::ShlEq(_)
        | BinOp::ShrEq(_)
        | BinOp::BitOrEq(_)
        | BinOp::BitAndEq(_)
        | BinOp::BitXorEq(_) => true,
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
