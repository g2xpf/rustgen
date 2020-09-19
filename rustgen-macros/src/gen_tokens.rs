use crate::env::Env;
use crate::object::eval_as_bool;
use crate::rustgen_ir::{
    BlockElse, BlockIf, Group, MacroBlock, MacroElse, MacroIf, RustgenIR, TokenBlock, TokenTree,
};
use proc_macro2::Delimiter;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::token::{Brace, Bracket, Paren};
use syn::Result;

pub trait GenTokens {
    type Output;
    fn gen_tokens(&self, env: &Env, token_stream: &mut TokenStream) -> Result<Self::Output>;
}

impl<T, S> GenTokens for Option<T>
where
    T: GenTokens<Output = S>,
{
    type Output = Option<S>;
    fn gen_tokens(&self, env: &Env, token_stream: &mut TokenStream) -> Result<Option<S>> {
        // Option t -> Result (Option t)
        match self {
            Some(t) => {
                let s = t.gen_tokens(env, token_stream)?;
                Ok(Some(s))
            }
            None => Ok(None),
        }
    }
}

impl GenTokens for RustgenIR {
    type Output = ();
    fn gen_tokens(&self, env: &Env, token_stream: &mut TokenStream) -> Result<Self::Output> {
        for token_block in &self.0 {
            token_block.gen_tokens(env, token_stream)?;
        }
        Ok(())
    }
}

impl GenTokens for TokenBlock {
    type Output = ();
    fn gen_tokens(&self, env: &Env, token_stream: &mut TokenStream) -> Result<()> {
        use TokenBlock::*;
        match self {
            TokenTree(token_tree) => token_tree.gen_tokens(env, token_stream)?,
            MacroIf(macro_if) => macro_if.gen_tokens(env, token_stream)?,
        }
        Ok(())
    }
}

impl GenTokens for TokenTree {
    type Output = ();
    fn gen_tokens(&self, env: &Env, token_stream: &mut TokenStream) -> Result<()> {
        use TokenTree::*;
        match self {
            Ident(ident) => ident.to_tokens(token_stream),
            Literal(lit) => lit.to_tokens(token_stream),
            Punct(punct) => punct.to_tokens(token_stream),
            Group(group) => group.gen_tokens(env, token_stream)?,
        }
        Ok(())
    }
}

impl GenTokens for Group {
    type Output = ();
    fn gen_tokens(&self, env: &Env, token_stream: &mut TokenStream) -> Result<()> {
        let mut error = None;
        match self.delimiter {
            Delimiter::Brace => {
                let brace = Brace { span: self.span };
                brace.surround(token_stream, |token_stream| {
                    self.ir
                        .gen_tokens(env, token_stream)
                        .unwrap_or_else(|err| error = Some(err));
                })
            }
            Delimiter::Bracket => {
                let bracket = Bracket { span: self.span };
                bracket.surround(token_stream, |token_stream| {
                    self.ir
                        .gen_tokens(env, token_stream)
                        .unwrap_or_else(|err| error = Some(err));
                });
            }
            Delimiter::Parenthesis => {
                let paren = Paren { span: self.span };
                paren.surround(token_stream, |token_stream| {
                    self.ir
                        .gen_tokens(env, token_stream)
                        .unwrap_or_else(|err| error = Some(err));
                });
            }
            Delimiter::None => unreachable!(),
        };

        if let Some(err) = error {
            Err(err)
        } else {
            Ok(())
        }
    }
}

impl GenTokens for MacroIf {
    type Output = ();
    fn gen_tokens(&self, env: &Env, token_stream: &mut TokenStream) -> Result<()> {
        let generated = self.if_item.gen_tokens(env, token_stream)?;
        if !generated {
            self.else_item.gen_tokens(env, token_stream)?;
        }
        Ok(())
    }
}

// depending on env
impl GenTokens for BlockIf {
    type Output = bool;
    fn gen_tokens(&self, env: &Env, token_stream: &mut TokenStream) -> Result<bool> {
        let value = env
            .search(&self.if_cond)
            .map_err(|err| err.into_syn_error())?;
        let should_generate = eval_as_bool(value);
        if should_generate {
            self.group.gen_tokens(env, token_stream)?;
        }
        Ok(should_generate)
    }
}

impl GenTokens for BlockElse {
    type Output = ();
    fn gen_tokens(&self, env: &Env, token_stream: &mut TokenStream) -> Result<()> {
        self.else_child.gen_tokens(env, token_stream)
    }
}

impl GenTokens for MacroElse {
    type Output = ();
    fn gen_tokens(&self, env: &Env, token_stream: &mut TokenStream) -> Result<()> {
        match self {
            MacroElse::If(macro_if) => macro_if.gen_tokens(env, token_stream),
            MacroElse::Block(macro_block) => macro_block.gen_tokens(env, token_stream),
        }
    }
}

impl GenTokens for MacroBlock {
    type Output = ();
    fn gen_tokens(&self, env: &Env, token_stream: &mut TokenStream) -> Result<()> {
        self.ir.gen_tokens(env, token_stream)
    }
}
