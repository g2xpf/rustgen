use crate::env::Env;
use crate::gen_tokens::GenTokens;
use crate::rustgen_ir::RustgenIR;
use proc_macro2::TokenStream;

#[derive(Debug)]
pub struct CodeGenerator {
    ir: RustgenIR,
    env: Env,
}

impl CodeGenerator {
    pub fn new(ir: RustgenIR, env: Env) -> Self {
        CodeGenerator { ir, env }
    }

    pub fn generate(self) -> TokenStream {
        let mut token_stream = TokenStream::new();
        if let Err(err) = self.ir.gen_tokens(&self.env, &mut token_stream) {
            return err.to_compile_error();
        }
        token_stream
    }
}
