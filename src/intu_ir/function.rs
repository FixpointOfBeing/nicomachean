use crate::intu_ir::basicblock::BasicBlock;
use crate::intu_ir::name::Name;
use crate::intu_ir::types::{TypeRef, Typed, Types};

#[derive(PartialEq, Clone, Debug, Hash)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: TypeRef,
    pub basic_blocks: Vec<BasicBlock>,
}

impl Typed for Function {
    fn get_type(&self, types: &Types) -> TypeRef {
        types.func_type(self.return_type.clone(), self.parameters.iter().map(|p| types.type_of(p)).collect())
    }
}

#[derive(PartialEq, Clone, Debug, Hash)]
pub struct FunctionDeclaration {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: TypeRef,
    pub alignment: u32,
}

#[derive(PartialEq, Clone, Debug, Hash)]
pub struct Parameter {
    pub name: Name,
    pub ty: TypeRef,
}

impl Typed for Parameter {
    fn get_type(&self, _types: &Types) -> TypeRef {
        self.ty.clone()
    }
}
