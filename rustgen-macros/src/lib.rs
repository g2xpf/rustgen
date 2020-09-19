mod code_generator;
mod env;
mod gen_tokens;
mod object;
mod parse;
mod rustgen_ir;

use code_generator::CodeGenerator;
use env::Env;
use rustgen_ir::RustgenIR;
use syn::parse_macro_input;

#[proc_macro]
pub fn gen(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let env = Env::new();
    let ir = parse_macro_input!(input as RustgenIR);
    let token_stream = CodeGenerator::new(ir, env).generate();
    token_stream.into()
}
