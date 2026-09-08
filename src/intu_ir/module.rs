use crate::intu_ir::constant::ConstantRef;
use crate::intu_ir::function::{Function, FunctionDeclaration};
use crate::intu_ir::name::Name;
use crate::intu_ir::show::Show;
use crate::intu_ir::types::{TypeRef, Typed, Types};
use std::io::Error;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Module {
    pub name: String,
    pub source_file_name: String,
    pub functions: Vec<Function>,
    pub func_declarations: Vec<FunctionDeclaration>,
    pub global_vars: Vec<GlobalVariable>,
    pub types: Types,
}

impl Module {
    pub fn type_of<T: Typed + ?Sized>(&self, t: &T) -> TypeRef {
        self.types.type_of(t)
    }

    pub fn get_func_by_name(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|func| func.name == name)
    }

    pub fn get_func_decl_by_name(&self, name: &str) -> Option<&FunctionDeclaration> {
        self.func_declarations.iter().find(|decl| decl.name == name)
    }

    pub fn get_global_var_by_name(&self, name: &Name) -> Option<&GlobalVariable> {
        self.global_vars.iter().find(|global| global.name == *name)
    }

    pub fn to_string(&self) -> String {
        self.show(&self.types)
    }

    pub fn print_to_file(&self, output: &PathBuf) -> Result<(), Error> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(output)?;
        let module_str = self.to_string();
        file.write_all(module_str.as_bytes())?;
        Ok(())
    }
}
pub type AddrSpace = u32;

#[derive(PartialEq, Clone, Debug, Hash)]
pub struct GlobalVariable {
    pub name: Name,
    pub is_constant: bool,
    pub ty: TypeRef,
    pub addr_space: AddrSpace,
    pub initializer: Option<ConstantRef>,
}

impl Typed for GlobalVariable {
    fn get_type(&self, _types: &Types) -> TypeRef {
        self.ty.clone()
    }
}
