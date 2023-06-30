#![feature(round_char_boundary)]
#![feature(iter_intersperse)]

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Ident};

mod kw {
    syn::custom_keyword!(declare);
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

struct KeyValue {
    key: Ident,
    colon: syn::Token![:],
    value: syn::Expr,
}

impl syn::parse::Parse for KeyValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(KeyValue {
            key: input.parse()?,
            colon: input.parse()?,
            value: input.parse()?,
        })
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
            arguments: arguments.parse_terminated(KeyValue::parse, syn::Token![,])?,
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
        return input.contains('_');
    }

    false
}

fn convert_expr_to_attr(expr: &syn::Expr) -> String {
    match expr {
        syn::Expr::Array(arr) => arr
            .elems
            .iter()
            .map(|elem| convert_expr_to_attr(elem))
            .intersperse(" ".to_string())
            .collect(),
        syn::Expr::Lit(lit) => lit.to_token_stream().to_string(),
        syn::Expr::Path(p) => p.to_token_stream().to_string(),
        _ => "".to_string(),
    }
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

                tokens.extend(quote! {
                    #let_token #ident = #struct_name {};
                });
            } else {
                let tag = name.to_string();
                tokens.extend(quote! {
                    #let_token #ident = document.create_element(#tag)?;
                });

                if let Some(args) = arguments {
                    for arg in &args.arguments {
                        let name = arg.key.to_string();
                        let string = convert_expr_to_attr(&arg.value);

                        tokens.extend(quote! {
                            #ident.set_attribute(#name, #string)?;
                        });
                    }
                }
            };

            match &body {
                ElementBody::Elements { brace_token, body } => {
                    if body.len() != 0 {
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

            for chunk in string.split_inclusive("}") {
                let mut text_and_pattern = chunk.split_inclusive("{");
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

            let mut bind = None;
            let mut subscribers = Vec::new();
            let mut vars = Vec::new();

            for value in &var_buf {
                if value.starts_with('$') {
                    if bind.is_none() {
                        let node_name = format!("_n{}", *index);
                        *index += 1;

                        let node_name = syn::Ident::new(&node_name, Span::call_site());
                        bind = Some(node_name);
                    }

                    let var_name = syn::Ident::new(&value[1..], Span::call_site());

                    vars.push(quote! {self.#var_name.value()});
                } else {
                    let var_name = syn::Ident::new(value, Span::call_site());
                    vars.push(quote! {#var_name});
                }
            }

            let format = quote! { let content = format!(#string, #(#vars),*); };

            for value in &var_buf {
                if value.starts_with('$') {
                    let var_name = syn::Ident::new(&value[1..], Span::call_site());

                    if let Some(name) = &bind {
                        subscribers.push(quote! {
                            let _self = self.clone();
                            _self.#var_name.subscribe(move |value| {
                                #format
                                #name.set_text_content(Some(&content));
                            });
                        });
                    }
                }
            }

            tokens.extend(quote! {
                #format
                let #bind = Box::new(document.create_text_node(&content));
                { #parent.append_child(&#bind.get_root_node())?; }
                #(#subscribers)*
            });
        }
    }

    tokens
}

#[proc_macro]
pub fn load(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let View {
        decl_token,
        name,
        elements,
        ..
    } = parse_macro_input!(input as View);

    let struct_token = syn::token::Struct {
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
        #impl_tok #name {
            pub fn on_init(self: Rc<Self>, document: Rc<tl_util::html::Document>, parent: &tl_util::html::Element) -> Result<(), JsValue> {
                // let Self { value } = self;
                let __body = document.body().expect("Unable to get document body");

                #(#tokens)*

                Ok(())
            }
        }
    };

    output.into()
}
