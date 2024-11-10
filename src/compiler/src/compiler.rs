use crate::parser::Program;

pub mod llvm;
pub mod jvm;

pub trait Compiler {
    type ExtendedData: Default;
    fn compile(&mut self, filename: &str, program: Program<Self::ExtendedData>) -> String;
}