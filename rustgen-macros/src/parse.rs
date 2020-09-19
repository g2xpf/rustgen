use crate::rustgen_ir::{
    BlockElse, BlockIf, ExprField, Group, MacroBlock, MacroElse, MacroIf, Member, RustgenIR,
    TokenBlock, TokenTree,
};
use proc_macro2::Delimiter;
use proc_macro2::TokenTree as TT;
use syn::braced;
use syn::parse::{Parse, ParseStream};
use syn::Result;
use syn::Token;

impl Parse for RustgenIR {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut token_blocks: Vec<TokenBlock> = vec![];
        while !input.is_empty() {
            let token_block = input.parse()?;
            token_blocks.push(token_block);
        }
        Ok(RustgenIR(token_blocks))
    }
}

#[test]
fn parse_rustgen_ir() {
    use quote::quote;
    use std::str::FromStr;
    let rustgen_ir: RustgenIR = parse(quote! {
        fn main() {}
    })
    .unwrap();
    println!("{:#?}", rustgen_ir);

    let rustgen_ir: RustgenIR = parse(
        proc_macro2::TokenStream::from_str(
            r#"
        #if a.b.c.d {
            fn main() {}
        }
    "#,
        )
        .unwrap(),
    )
    .unwrap();
    println!("{:#?}", rustgen_ir);

    let rustgen_ir: RustgenIR = parse(
        proc_macro2::TokenStream::from_str(
            r#"
        #if a.b.c.d {
            fn main() {}
        } #else {
            fn main() {}
        }
    "#,
        )
        .unwrap(),
    )
    .unwrap();
    println!("{:#?}", rustgen_ir);

    let rustgen_ir = parse::<RustgenIR>(
        proc_macro2::TokenStream::from_str(
            r#"
        #if a.b.c.d {
            fn main() {}
        } #else #if {
            fn main() {}
        }
    "#,
        )
        .unwrap(),
    );
    println!("{:#?}", rustgen_ir.err().unwrap().to_compile_error());
}

fn parse<T: Parse>(token_stream: proc_macro2::TokenStream) -> Result<T> {
    syn::parse2(token_stream)
}

impl Parse for TokenBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead1 = input.lookahead1();
        if lookahead1.peek(Token![#]) {
            input.parse().map(TokenBlock::MacroIf)
        } else {
            input.parse().map(TokenBlock::TokenTree)
        }
    }
}

#[test]
fn parse_group() {
    use quote::quote;
    let _: Group = parse(quote! {
        {
            fn hi
        }
    })
    .unwrap();
}

impl Parse for Group {
    fn parse(input: ParseStream) -> Result<Self> {
        input.step(|cursor| {
            for delimiter in &[Delimiter::Parenthesis, Delimiter::Brace, Delimiter::Bracket] {
                if let Some((inside, span, rest)) = cursor.group(*delimiter) {
                    let ir = parse(inside.token_stream())?;
                    let group = Group {
                        delimiter: *delimiter,
                        ir,
                        span,
                    };
                    return Ok((group, rest));
                }
            }
            Err(cursor.error("expected group token"))
        })
    }
}

impl Parse for TokenTree {
    fn parse(input: ParseStream) -> Result<Self> {
        input.step(|cursor| {
            if let Some((tt, next)) = cursor.token_tree() {
                let tt = match tt {
                    TT::Punct(punct) => TokenTree::Punct(punct),
                    TT::Ident(ident) => TokenTree::Ident(ident),
                    TT::Literal(lit) => TokenTree::Literal(lit),
                    TT::Group(group) => {
                        let delimiter = group.delimiter();
                        let ir = parse(group.stream())?;
                        let span = group.span();
                        TokenTree::Group(Group {
                            delimiter,
                            ir,
                            span,
                        })
                    }
                };
                return Ok((tt, next));
            }
            Err(cursor.error("unexpected <eof>"))
        })
    }
}

impl Parse for MacroIf {
    fn parse(input: ParseStream) -> Result<Self> {
        let if_item = input.parse()?;
        let lookahead1 = input.lookahead1();
        let else_item = if lookahead1.peek(Token![#]) {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(MacroIf { if_item, else_item })
    }
}

impl Parse for BlockIf {
    fn parse(input: ParseStream) -> Result<Self> {
        let pound_token = input.parse()?;
        let if_token = input.parse()?;
        let if_cond = input.parse()?;
        let group = input.parse()?;
        Ok(BlockIf {
            pound_token,
            if_cond,
            if_token,
            group,
        })
    }
}

impl Parse for MacroBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let brace_token = braced!(content in input);
        let ir = content.parse()?;
        Ok(MacroBlock { brace_token, ir })
    }
}

impl Parse for BlockElse {
    fn parse(input: ParseStream) -> Result<Self> {
        let pound_token = input.parse()?;
        let else_token = input.parse()?;
        let else_child = input.parse()?;
        Ok(BlockElse {
            pound_token,
            else_child,
            else_token,
        })
    }
}

impl Parse for MacroElse {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead1 = input.lookahead1();
        if lookahead1.peek(Token![#]) {
            input.parse().map(MacroElse::If)
        } else {
            input.parse().map(MacroElse::Block)
        }
    }
}

impl Parse for ExprField {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;
        let lookahead1 = input.lookahead1();
        if lookahead1.peek(Token![.]) {
            let member = Some(input.parse()?);
            Ok(ExprField { ident, member })
        } else {
            Ok(ExprField {
                ident,
                member: None,
            })
        }
    }
}

impl Parse for Member {
    fn parse(input: ParseStream) -> Result<Self> {
        let dot_token = input.parse()?;
        let field = input.parse()?;
        Ok(Member { dot_token, field })
    }
}
