use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, BinOp, Ident};

mod expr;
mod syn_macros;

mod kw {
    syn::custom_keyword!(declare);
}

mod punc {
    syn::custom_punctuation!(FnBind, @);
}

enum ElementBody {
    Empty {
        semi: syn::Token![;],
    },
    Elements {
        brace_token: syn::token::Brace,
        body: Vec<Element>,
    },
}

impl syn::parse::Parse for ElementBody {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let la = input.lookahead1().peek(syn::Token![;]);
        if la {
            Ok(ElementBody::Empty {
                semi: input.parse()?,
            })
        } else {
            let content;

            let brace = syn::braced!(content in input);

            let mut body = Vec::new();

            while !content.is_empty() {
                body.push(content.parse::<Element>()?);
            }

            Ok(ElementBody::Elements {
                brace_token: brace,
                body,
            })
        }
    }
}

enum Value {
    FnBind {
        bind_tok: punc::FnBind,
        ident: syn::Ident,
    },
    Expr(expr::CoreExpr),
    Stmt {
        group: syn::token::Brace,
        stmt: expr::CoreExpr,
    },
}

impl Value {
    pub fn as_fn_bind(&self) -> &syn::Ident {
        match self {
            Self::FnBind { ident, .. } => ident,
            _ => panic!("Expected fn bind"),
        }
    }

    pub fn as_expr(&self) -> &expr::CoreExpr {
        match self {
            Self::Expr(expr) => expr,
            _ => panic!("Expected Expr"),
        }
    }
}

impl syn::parse::Parse for Value {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(punc::FnBind) {
            Ok(Value::FnBind {
                bind_tok: input.parse()?,
                ident: input.parse()?,
            })
        } else if input.peek(syn::token::Brace) {
            let content;
            Ok(Value::Stmt {
                group: syn::braced!(content in input),
                stmt: content.parse()?,
            })
        } else {
            Ok(Value::Expr(input.parse()?))
        }
    }
}

struct KeyValue {
    key: Option<(Ident, syn::Token![:])>,
    value: expr::CoreExpr,
}

impl syn::parse::Parse for KeyValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek2(syn::Token![:]) {
            Ok(KeyValue {
                key: Some((input.parse()?, input.parse()?)),
                value: input.parse()?,
            })
        } else {
            Ok(KeyValue {
                key: None,
                value: input.parse()?,
            })
        }
    }
}

struct Arguments {
    parens: syn::token::Paren,
    arguments: syn::punctuated::Punctuated<KeyValue, syn::Token![,]>,
}

impl syn::parse::Parse for Arguments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let arguments;
        Ok(Arguments {
            parens: syn::parenthesized!(arguments in input),
            arguments: arguments.parse_terminated(KeyValue::parse)?,
        })
    }
}

enum Element {
    Tag {
        name: Ident,
        arguments: Option<Arguments>,
        body: ElementBody,
    },
    Text(syn::LitStr),
}

impl Element {
    pub fn is_view(&self) -> bool {
        match self {
            Self::Tag { name, .. } => is_pascal(&name.to_string()),
            _ => false,
        }
    }
}

impl syn::parse::Parse for Element {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek2(syn::token::Paren)
            || input.peek2(syn::token::Brace)
            || input.peek2(syn::Token![;])
        {
            let name = input.parse()?;

            let arguments = if input.peek(syn::token::Paren) {
                Some(input.parse()?)
            } else {
                None
            };

            Ok(Element::Tag {
                name,
                arguments,
                body: input.parse()?,
            })
        } else {
            // For some reason, calling source_text invalidates span
            // let span = input.cursor().span();
            // let ts = input
            //     .step(|cursor| {
            //         let mut rest = *cursor;

            //         let mut ts = TokenStream::new();

            //         while let Some((tt, next)) = rest.token_tree() {
            //             rest = next;
            //             // let text = tt.span().source_text().unwrap();
            //             ts.extend(tt.into_token_stream());
            //         }
            //         Ok((ts, rest))
            //     })
            //     .expect("Unable to step");
            // let raw_text = span.clone().source_text().unwrap();

            // let string = syn::LitStr::new(&ts.to_string(), span);
            // syn::parse_quote!()

            Ok(Element::Text(input.parse()?))
        }
    }
}

struct View {
    decl_token: kw::declare,
    name: syn::Ident,
    semi: syn::Token![;],

    elements: Vec<Element>,
}

impl syn::parse::Parse for View {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(View {
            decl_token: input.parse()?,
            name: input.parse()?,
            semi: input.parse()?,

            elements: {
                let mut elements = Vec::new();

                while !input.is_empty() {
                    elements.push(input.parse::<Element>()?);
                }
                elements
            },
        })
    }
}

fn is_pascal(input: &str) -> bool {
    if let Some('A'..='Z') = input.chars().next() {
        return !input.contains('_');
    }

    false
}

fn convert_expr_to_attr(expr: &syn::Expr) -> String {
    match expr {
        syn::Expr::Array(arr) => arr
            .elems
            .iter()
            .map(convert_expr_to_attr)
            .collect::<Vec<_>>().join(" "),
        syn::Expr::Lit(lit) => lit.to_token_stream().to_string(),
        syn::Expr::Path(p) => p.to_token_stream().to_string(),
        _ => "".to_string(),
    }
}

fn op_to_func_name(op: &BinOp) -> syn::Ident {
    let name = match op {
        BinOp::AddEq(_) => "add",
        BinOp::SubEq(_) => "sub",
        BinOp::MulEq(_) => "mul",
        BinOp::DivEq(_) => "div",
        BinOp::RemEq(_) => "rem",
        BinOp::ShrEq(_) => "shr",
        BinOp::ShlEq(_) => "shl",
        BinOp::BitAndEq(_) => "bitand",
        BinOp::BitOrEq(_) => "bitor",
        BinOp::BitXorEq(_) => "bitxor",
        _ => panic!("Unsupported operator"),
    };

    syn::Ident::new(name, Span::call_site())
}

fn generate_expr(expr: &expr::CoreExpr, view_param: bool) -> TokenStream {
    match expr {
        expr::CoreExpr::Expr(ex) => ex.to_token_stream(),
        expr::CoreExpr::FnBind(binding) => {
            // dot = Some(syn::token::Dot { spans: binding.bind_token.spans });
            let var_name = &binding.ident;

            quote! {}
        }
        expr::CoreExpr::StateBind(binding) => {
            let dot = Some(syn::token::Dot {
                spans: binding.bind_token.spans,
            });
            let var_name = &binding.ident;

            if view_param {
                quote! { self #dot #var_name.bind() }
            } else {
                quote! { _self #dot #var_name.value() }
            }
        }
        expr::CoreExpr::Assignment(expr::Assignment { left, op, right }) => {
            match (left.as_ref(), right.as_ref()) {
                (expr::CoreExpr::StateBind(bind), expr::CoreExpr::Expr(exp)) => {
                    let op = op_to_func_name(op);
                    let var_name = &bind.ident;

                    quote! { _self.#var_name.value_mut().#op(#exp) }
                }
                _ => panic!("Unexpected assignment"),
            }
        }
        _ => panic!("Unexpected expression"),
        // expr::CoreExpr::Closure(expr::Closure { expr, .. }) => {
        //     get_event_from_stmt(expr, &name, &ident)
        // }
    }
}

fn get_event_from_stmt(
    stmt: &expr::CoreExpr,
    event_name: &syn::LitStr,
    element: &syn::Ident,
) -> TokenStream {
    let expr = generate_expr(stmt, false);

    quote! {
        {
            let _self = self.clone();
            let cb: Closure<dyn FnMut(Event)> = Closure::new(move |_| {
                let _self = Rc::clone(&_self);
                {
                    #expr;
                }
            });

            #element.add_event_listener_with_callback(#event_name, &cb.as_ref().unchecked_ref())?;
            cb.forget();
        }
    }
}

fn get_event(
    ident: &syn::Ident,
    event_name: &syn::LitStr,
    element: &syn::Ident,
    tok_span: Span,
) -> TokenStream {
    let path_string = ident.to_string();
    let func_name = &path_string[..];
    let func_ident = syn::Ident::new(func_name, Span::call_site());
    // syn::LitChar::new(value, span);

    let dot = syn::token::Dot { spans: [tok_span] };

    quote! {
        {
            let _self = self.clone();
            let cb: Closure<dyn FnMut(Event)> = Closure::new(move |_| {
                let _self = Rc::clone(&_self);
                _self #dot #func_ident()
            });

            #element.add_event_listener_with_callback(#event_name, &cb.as_ref().unchecked_ref())?;
            cb.forget();
        }
    }
}

// const
lazy_static::lazy_static! {
    static ref EVENTS: HashSet<&'static str> = HashSet::from_iter([
        "click"
    ]);
}

fn walk_elements(index: &mut usize, parent: &Ident, element: &Element) -> TokenStream {
    let mut tokens = TokenStream::new();

    match element {
        Element::Tag {
            name,
            arguments,
            body,
        } => {
            let var_name = format!("_e{}", *index);
            *index += 1;

            let ident = syn::Ident::new(&var_name, proc_macro2::Span::call_site());
            let let_token = syn::token::Let { span: name.span() };

            if element.is_view() {
                let struct_name = &name;

                let args = if let Some(args) = arguments {
                    let mut params = Vec::new();
                    for arg in &args.arguments {
                        params.push(generate_expr(&arg.value, true))
                    }
                    params
                } else {
                    Vec::new()
                };

                tokens.extend(quote! {
                    #let_token #ident = Rc::new(#struct_name::new(#(#args),*));
                    #ident.on_init(document.clone(), &#parent)?;
                });

                return tokens;
            } else {
                let tag = name.to_string();
                tokens.extend(quote! {
                    #let_token #ident = document.create_element(#tag)?;
                });

                if let Some(args) = arguments {
                    for arg in &args.arguments {
                        let name = arg.key.as_ref().expect("Expected named argument for element").0.to_string();

                        if EVENTS.contains(name.as_str()) {
                            let name = syn::LitStr::new(name.as_str(), Span::call_site());

                            match &arg.value {
                                expr::CoreExpr::FnBind(expr::FnBind {
                                    bind_token,
                                    ident: bind_ident,
                                }) => {
                                    let event_value =
                                        get_event(bind_ident, &name, &ident, bind_token.spans[0]);
                                    tokens.extend(event_value);
                                }
                                expr::CoreExpr::Closure(expr::Closure { expr, .. }) => {
                                    let event_value = get_event_from_stmt(expr, &name, &ident);
                                    tokens.extend(event_value);
                                }
                                _ => panic!("Only Expected Function bind at the moment"),
                            }
                        } else {
                            // let expr = arg.value.as_expr();
                            match &arg.value {
                                expr::CoreExpr::Expr(ex) => {
                                    let string = convert_expr_to_attr(ex);

                                    tokens.extend(quote! {
                                        #ident.set_attribute(#name, #string)?;
                                    });
                                }
                                _ => panic!("Unexpected expression for attribute"),
                            }
                        }
                    }
                }
            };

            match &body {
                ElementBody::Elements { brace_token, body } => {
                    if !body.is_empty() {
                        brace_token.surround(&mut tokens, |body_tokens| {
                            for element in body {
                                let sub_tokens = walk_elements(index, &ident, element);

                                body_tokens.extend(sub_tokens);
                            }
                        })
                    }
                }
                _ => (),
            }

            tokens.extend(quote! {
                #parent.append_child(&#ident)?;
            });
        }
        Element::Text(lit_str) => {
            let string = lit_str.value();
            let mut count = 0;
            let mut buf = String::new();
            let mut var_buf = Vec::new();

            for chunk in string.split_inclusive('}') {
                let mut text_and_pattern = chunk.split_inclusive('{');
                let text = text_and_pattern.next().unwrap();
                buf.push_str(text);
                count += text.len();

                if let Some(p) = text_and_pattern.next() {
                    if let Some(' ' | '}') = p.chars().next() {
                        buf.push_str(p);
                        count += p.len();
                        continue;
                    }

                    let mut var_and_format = p.split(':');

                    let var = var_and_format.next().unwrap();

                    if let Some(p) = var_and_format.next() {
                        if !var.is_empty() {
                            // let var = syn::Ident::new(var, proc_macro2::Span::call_site());
                            var_buf.push(var);
                        }

                        buf.push(':');
                        buf.push_str(p);
                    } else {
                        let mut var_and_curly = var.split_terminator('}');

                        let var = var_and_curly.next().unwrap();
                        // let var = syn::Ident::new(var, proc_macro2::Span::call_site());

                        var_buf.push(var);

                        buf.push('}');

                        assert!(var_and_curly.next().is_none());
                    }

                    assert!(var_and_format.next().is_none());
                }
            }

            let string = syn::LitStr::new(&buf, lit_str.span());

            let bind = {
                let node_name = format!("_n{}", *index);
                *index += 1;

                let node_name = syn::Ident::new(&node_name, Span::call_site());
                node_name
            };

            let mut subscribers = Vec::new();
            let mut vars = Vec::new();
            let mut re_fmt_vars = Vec::new();

            for value in &var_buf {
                if value.starts_with('$') {
                    let var_name = syn::Ident::new(&value[1..], Span::call_site());
                    vars.push(quote! {self.#var_name.value()});

                    re_fmt_vars.push(quote! {_self.#var_name.value()});
                } else {
                    let var_name = syn::Ident::new(value, Span::call_site());
                    vars.push(quote! {#var_name});
                    re_fmt_vars.push(quote! {#var_name});
                }
            }

            let format = quote! { let content = format!(#string, #(#vars),*); };
            let re_format = quote! { let content = format!(#string, #(#re_fmt_vars),*); };

            for value in &var_buf {
                if value.starts_with('$') {
                    let var_name = syn::Ident::new(&value[1..], Span::call_site());

                    let new_name =
                        syn::Ident::new(&format!("{}_clone", bind.to_string()), Span::call_site());

                    subscribers.push(quote! {
                        let _self = self.clone();
                        let #new_name = #bind.clone();
                        self.#var_name.subscribe(move |value| {
                            #re_format
                            #new_name.set_text_content(Some(&content));
                        });
                    });
                }
            }

            tokens.extend(quote! {
                #format
                let #bind = document.create_text_node(&content);
                { #parent.append_child(&#bind.get_root_node())?; }
                #(#subscribers)*
            });
        }
    }

    tokens
}

#[proc_macro]
pub fn view(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let View {
        decl_token,
        name,
        elements,
        ..
    } = parse_macro_input!(input as View);

    let _struct_token = syn::token::Struct {
        span: decl_token.span,
    };

    let mut index = 0;
    let mut tokens = Vec::new();

    for element in &elements {
        let sub_tokens = walk_elements(
            &mut index,
            &Ident::new_raw("__body", proc_macro2::Span::call_site()),
            element,
        );

        tokens.push(sub_tokens);
    }

    let impl_tok = syn::token::Impl {
        span: decl_token.span,
    };

    let output = quote! {
        use std::rc::Rc;

        use tsz::State;
        use wasm_bindgen::prelude::*;
        use web_sys::Event;

        #impl_tok #name {
            pub fn on_init(self: Rc<Self>, document: Rc<tsz::html::Document>, parent: &tsz::html::Element) -> Result<(), JsValue> {
                // let Self { value } = self;
                let __body = document.body().expect("Unable to get document body");

                #(#tokens);*

                Ok(())
            }
        }
    };

    output.into()
}
