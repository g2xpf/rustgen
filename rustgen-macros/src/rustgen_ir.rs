use proc_macro2::{Delimiter, Ident, Literal, Punct, Span};
use syn::token::Brace;
use syn::Token;

macro_rules! impl_enum_debug {
   ($ty:ident, $($variant:ident),*) => {
        impl std::fmt::Debug for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    $(
                        Self::$variant(v) => v.fmt(f),
                    )*
                }
            }
        }
    }
}

impl_enum_debug!(TokenBlock, TokenTree, MacroIf);
impl_enum_debug!(TokenTree, Punct, Ident, Literal, Group);
impl_enum_debug!(MacroElse, If, Block);

#[derive(Debug, Clone)]
pub struct RustgenIR(pub Vec<TokenBlock>);

#[derive(Clone)]
pub enum TokenBlock {
    TokenTree(TokenTree),
    MacroIf(MacroIf),
}

#[derive(Debug, Clone)]
pub struct Group {
    pub delimiter: Delimiter,
    pub ir: Box<RustgenIR>,
    pub span: Span,
}

#[derive(Clone)]
pub enum TokenTree {
    Ident(Ident),
    Literal(Literal),
    Punct(Punct),
    Group(Group),
}

#[derive(Debug, Clone)]
pub struct MacroIf {
    pub if_item: BlockIf,
    pub else_item: Option<BlockElse>,
}

#[derive(Debug, Clone)]
pub struct BlockIf {
    pub pound_token: Token![#],
    pub if_token: Token![if],
    pub if_cond: ExprField,
    pub group: Box<MacroBlock>,
}

#[derive(Debug, Clone)]
pub struct MacroBlock {
    pub brace_token: Brace,
    pub ir: Box<RustgenIR>,
}

#[derive(Debug, Clone)]
pub struct BlockElse {
    pub pound_token: Token![#],
    pub else_token: Token![else],
    pub else_child: MacroElse,
}

#[derive(Clone)]
pub enum MacroElse {
    If(Box<MacroIf>),
    Block(Box<MacroBlock>),
}

#[derive(Debug, Clone)]
pub struct ExprField {
    pub ident: Ident,
    pub member: Option<Member>,
}

impl ExprField {
    pub fn name(&self) -> String {
        self.ident.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Member {
    pub dot_token: Token![.],
    pub field: Box<ExprField>,
}
