use crate::glacier_parser::ast::Program;

pub trait CodeGen {
    type ResType;
    type OptionType;
    fn generate<'a>(&mut self, program: &'a Program, options: Self::OptionType) -> Self::ResType;
}

pub mod js;
