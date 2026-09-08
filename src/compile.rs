use std::path::{Path, PathBuf};

use crate::{
    a_normal_form::anf_convert,
    closure_conversion::{ClosFnDef, closure_convert},
    emit_llvm::emit_module,
    explicate_control::explicate_control_convert,
    uniquify::uniquify_convert,
};

pub fn compile_file(file_path: &Path, output: &Option<PathBuf>) -> Result<(), String> {
    use lalrpop_util::lalrpop_mod;
    lalrpop_mod!(pub parser);
    use crate::typechecker::typecheck;
    use std::fs::read_to_string;

    let source = read_to_string(file_path).map_err(|e| e.to_string())?;
    let source_file_name = file_path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid file name")?
        .to_string();
    let module_name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid file name")?
        .to_string();

    // Paring
    let expr = parser::ExprParser::new().parse(&source).map_err(|e| e.to_string())?;

    // Type Checking
    let (return_ty, typed_expr) = typecheck(*expr).map_err(|e| e.to_string())?;

    // Uniquify
    let typed_expr = uniquify_convert(typed_expr);

    // ANF Conversion
    let anf_expr = anf_convert(typed_expr);

    // Closure Conversion
    let clos_prog = closure_convert(anf_expr);

    // Explicate Control
    let body_ctail = explicate_control_convert(clos_prog.body);
    let fn_ctails: Vec<(ClosFnDef, _)> = clos_prog
        .fn_defs
        .iter()
        .map(|d| (d.clone(), explicate_control_convert(d.body.clone())))
        .collect();

    // Emit LLVM
    let module = emit_module(body_ctail, return_ty, module_name, source_file_name, &fn_ctails);

    match output {
        Some(output_path) => {
            module.print_to_file(output_path).map_err(|e| e.to_string())?;
        },
        None => {
            println!("{:?}", module);
            println!("{}", module.to_string());
        },
    }
    Ok(())
}
