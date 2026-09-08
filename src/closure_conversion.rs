use std::collections::HashSet;

use crate::a_normal_form::{AExpr, AnfExpr, CompExpr};
use crate::gensym::Gensym;
use crate::syntax::{BinOp, Ident, Type, UnaryOp};

#[derive(Debug, Clone, PartialEq)]
pub enum ClosCompExpr {
    Atom(AExpr),
    BinOp(BinOp, AExpr, AExpr),
    UnaryOp(UnaryOp, AExpr),
    App(AExpr, Vec<AExpr>),
    If(AExpr, Box<ClosExpr>, Box<ClosExpr>),
    MakeClosure(AExpr, Vec<AExpr>, Type),
    Project(AExpr, usize, Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClosExpr {
    Complex(ClosCompExpr),
    Let(Ident, ClosCompExpr, Box<ClosExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosFnDef {
    pub name: Ident,
    pub env_param: Ident,
    pub params: Vec<(Ident, Type)>,
    pub return_type: Type,
    pub body: ClosExpr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClosProgram {
    pub fn_defs: Vec<ClosFnDef>,
    pub body: ClosExpr,
}

fn free_vars_comp(comp: &CompExpr, bound: &mut HashSet<Ident>, free: &mut HashSet<Ident>) {
    match comp {
        CompExpr::Atom(aexpr) => free_vars_aexpr(aexpr, bound, free),
        CompExpr::BinOp(_, l, r) => {
            free_vars_aexpr(l, bound, free);
            free_vars_aexpr(r, bound, free);
        },
        CompExpr::UnaryOp(_, a) => free_vars_aexpr(a, bound, free),
        CompExpr::App(fn_a, args) => {
            free_vars_aexpr(fn_a, bound, free);
            for a in args {
                free_vars_aexpr(a, bound, free);
            }
        },
        CompExpr::If(cond, thn, els) => {
            free_vars_aexpr(cond, bound, free);
            free_vars_anf(thn, bound, free);
            free_vars_anf(els, bound, free);
        },
        CompExpr::Lambda(params, _, body) => {
            let old_bound: HashSet<_> = bound.clone();
            for (name, _) in params {
                bound.insert(name.clone());
            }
            free_vars_anf(body, bound, free);
            *bound = old_bound;
        },
    }
}

fn free_vars_anf(anf: &AnfExpr, bound: &mut HashSet<Ident>, free: &mut HashSet<Ident>) {
    match anf {
        AnfExpr::Complex(comp) => free_vars_comp(comp, bound, free),
        AnfExpr::Let(name, rhs, body) => {
            free_vars_comp(rhs, bound, free);
            bound.insert(name.clone());
            free_vars_anf(body, bound, free);
            bound.remove(name);
        },
        AnfExpr::LetRec(name, params, _, body, rest) => {
            bound.insert(name.clone());
            for (pname, _) in params {
                bound.insert(pname.clone());
            }
            free_vars_anf(body, bound, free);
            for (pname, _) in params {
                bound.remove(pname);
            }
            free_vars_anf(rest, bound, free);
            bound.remove(name);
        },
    }
}

fn free_vars_aexpr(aexpr: &AExpr, _bound: &HashSet<Ident>, free: &mut HashSet<Ident>) {
    if let AExpr::Var(name, _) = aexpr {
        if !_bound.contains(name) {
            free.insert(name.clone());
        }
    }
}

fn free_vars_of_anf(anf: &AnfExpr) -> Vec<(Ident, Type)> {
    let mut bound = HashSet::new();
    let mut free = HashSet::new();
    free_vars_anf(anf, &mut bound, &mut free);

    let mut pairs: Vec<(Ident, Type)> = Vec::new();
    for name in free {
        let ty = infer_type_from_anf(anf, &name);
        pairs.push((name, ty));
    }
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    pairs
}

fn infer_type_from_anf(anf: &AnfExpr, target: &str) -> Type {
    let mut result: Option<Type> = None;
    search_anf(anf, target, &mut result);

    fn search_anf(anf: &AnfExpr, target: &str, result: &mut Option<Type>) {
        if result.is_some() {
            return;
        }
        match anf {
            AnfExpr::Complex(comp) => search_comp(comp, target, result),
            AnfExpr::Let(_, rhs, body) => {
                search_comp(rhs, target, result);
                if result.is_none() {
                    search_anf(body, target, result);
                }
            },
            AnfExpr::LetRec(name, params, ret_ty, body, rest) => {
                if name == target {
                    let fn_ty = params
                        .iter()
                        .rfold(ret_ty.clone(), |acc, (_, pty)| Type::Arrow(Box::new(pty.clone()), Box::new(acc)));
                    *result = Some(fn_ty);
                    return;
                }
                search_anf(body, target, result);
                if result.is_none() {
                    search_anf(rest, target, result);
                }
            },
        }
    }

    fn search_comp(comp: &CompExpr, target: &str, result: &mut Option<Type>) {
        if result.is_some() {
            return;
        }
        match comp {
            CompExpr::Atom(aexpr) => {
                if let AExpr::Var(name, ty) = aexpr {
                    if name == target {
                        *result = Some(ty.clone());
                    }
                }
            },
            CompExpr::BinOp(_, l, r) => {
                search_aexpr(l, target, result);
                if result.is_none() {
                    search_aexpr(r, target, result);
                }
            },
            CompExpr::UnaryOp(_, a) => {
                search_aexpr(a, target, result);
            },
            CompExpr::App(f, args) => {
                search_aexpr(f, target, result);
                for a in args {
                    if result.is_some() {
                        return;
                    }
                    search_aexpr(a, target, result);
                }
            },
            CompExpr::If(_, thn, els) => {
                search_anf(thn, target, result);
                if result.is_none() {
                    search_anf(els, target, result);
                }
            },
            CompExpr::Lambda(params, _, body) => {
                for (pname, pty) in params {
                    if pname == target {
                        *result = Some(pty.clone());
                        return;
                    }
                }
                search_anf(body, target, result);
            },
        }
    }

    fn search_aexpr(aexpr: &AExpr, target: &str, result: &mut Option<Type>) {
        if let AExpr::Var(name, ty) = aexpr {
            if name == target {
                *result = Some(ty.clone());
            }
        }
    }

    result.unwrap_or(Type::Unit)
}

fn compile_comp(comp: CompExpr, bound: &HashSet<Ident>, fn_defs: &mut Vec<ClosFnDef>, gs: &mut Gensym) -> ClosCompExpr {
    match comp {
        CompExpr::Atom(a) => ClosCompExpr::Atom(a),
        CompExpr::BinOp(op, l, r) => ClosCompExpr::BinOp(op, l, r),
        CompExpr::UnaryOp(op, a) => ClosCompExpr::UnaryOp(op, a),
        CompExpr::App(fn_a, args) => ClosCompExpr::App(fn_a, args),
        CompExpr::If(cond, thn, els) => {
            let then_clos = compile_anf(*thn, bound, fn_defs, gs);
            let else_clos = compile_anf(*els, bound, fn_defs, gs);
            ClosCompExpr::If(cond, Box::new(then_clos), Box::new(else_clos))
        },
        CompExpr::Lambda(params, ret_ty, body) => {
            let fn_name = gs.fresh_with_prefix("lambda$");
            let env_param_name = gs.fresh_with_prefix("env$");

            let mut lambda_bound: HashSet<Ident> = HashSet::new();
            for (pname, _) in params.clone() {
                lambda_bound.insert(pname);
            }
            lambda_bound.insert(env_param_name.clone());

            let (conv_body, free_vars) =
                convert_closure_body(*body, bound, &lambda_bound, &env_param_name, fn_defs, gs);

            let fn_def = ClosFnDef {
                name: fn_name.clone(),
                env_param: env_param_name,
                params: params.clone(),
                return_type: ret_ty.clone(),
                body: conv_body,
            };
            fn_defs.push(fn_def);

            let fn_ptr_atom = AExpr::Var(fn_name, Type::Arrow(Box::new(Type::Unit), Box::new(ret_ty.clone())));

            let mut captured_atoms: Vec<AExpr> = Vec::new();
            for (fv_name, fv_ty) in &free_vars {
                captured_atoms.push(AExpr::Var(fv_name.clone(), fv_ty.clone()));
            }

            let original_fn_ty = params
                .iter()
                .rfold(ret_ty.clone(), |acc, (_, pty)| Type::Arrow(Box::new(pty.clone()), Box::new(acc)));

            ClosCompExpr::MakeClosure(fn_ptr_atom, captured_atoms, original_fn_ty)
        },
    }
}

fn convert_closure_body(
    body: AnfExpr,
    outer_bound: &HashSet<Ident>,
    lambda_bound: &HashSet<Ident>,
    env_param_name: &str,
    fn_defs: &mut Vec<ClosFnDef>,
    gs: &mut Gensym,
) -> (ClosExpr, Vec<(Ident, Type)>) {
    let mut free_set = HashSet::new();
    let mut bound_mut = lambda_bound.clone();
    free_vars_anf(&body, &mut bound_mut, &mut free_set);

    let mut free_vars: Vec<(Ident, Type)> = Vec::new();
    for name in &free_set {
        let ty = infer_type_from_anf(&body, name);
        free_vars.push((name.clone(), ty));
    }
    free_vars.sort_by(|a, b| a.0.cmp(&b.0));

    let _env_ty = free_vars.iter().fold(Type::Unit, |acc, (_, ty)| {
        if matches!(acc, Type::Unit) { ty.clone() } else { Type::Arrow(Box::new(ty.clone()), Box::new(acc)) }
    });

    let env_field_index: std::collections::HashMap<Ident, usize> = free_vars
        .iter()
        .enumerate()
        .map(|(i, (name, _))| (name.clone(), i))
        .collect();

    fn compile_anf_with_env(
        anf: AnfExpr,
        bound: &HashSet<Ident>,
        lambda_bound: &HashSet<Ident>,
        env_param: &str,
        env_field_index: &std::collections::HashMap<Ident, usize>,
        fn_defs: &mut Vec<ClosFnDef>,
        gs: &mut Gensym,
    ) -> ClosExpr {
        match anf {
            AnfExpr::Complex(comp) => {
                let clos_comp =
                    compile_comp_with_env(comp, bound, lambda_bound, env_param, env_field_index, fn_defs, gs);
                ClosExpr::Complex(clos_comp)
            },
            AnfExpr::Let(name, rhs, body) => {
                let rhs_clos = compile_comp_with_env(rhs, bound, lambda_bound, env_param, env_field_index, fn_defs, gs);
                let mut new_bound = bound.clone();
                new_bound.insert(name.clone());
                let body_clos =
                    compile_anf_with_env(*body, &new_bound, lambda_bound, env_param, env_field_index, fn_defs, gs);
                ClosExpr::Let(name.clone(), rhs_clos, Box::new(body_clos))
            },
            AnfExpr::LetRec(name, params, ret_ty, body, rest) => {
                let mut rec_lambda_bound = lambda_bound.clone();
                rec_lambda_bound.insert(name.clone());
                for (pname, _) in params.clone() {
                    rec_lambda_bound.insert(pname);
                }

                let (fbody_clos, fv_rec) =
                    convert_closure_body(*body, bound, &rec_lambda_bound, env_param, fn_defs, gs);

                let fn_inner_name = gs.fresh_with_prefix("recur$");

                let fn_def = ClosFnDef {
                    name: fn_inner_name.clone(),
                    env_param: env_param.to_string(),
                    params: params.clone(),
                    return_type: ret_ty.clone(),
                    body: fbody_clos,
                };
                fn_defs.push(fn_def);

                let fn_ptr_atom =
                    AExpr::Var(fn_inner_name.clone(), Type::Arrow(Box::new(Type::Unit), Box::new(ret_ty.clone())));

                let mut captured_atoms: Vec<AExpr> = Vec::new();
                for (fv_name, fv_ty) in &fv_rec {
                    let atom = compile_var_with_env(fv_name, fv_ty, bound, lambda_bound, env_param, env_field_index);
                    captured_atoms.push(atom);
                }

                let original_fn_ty = params
                    .iter()
                    .rfold(ret_ty.clone(), |acc, (_, pty)| Type::Arrow(Box::new(pty.clone()), Box::new(acc)));

                let mut new_bound = bound.clone();
                new_bound.insert(name.clone());
                let rest_clos =
                    compile_anf_with_env(*rest, &new_bound, lambda_bound, env_param, env_field_index, fn_defs, gs);

                let make_clos = ClosCompExpr::MakeClosure(fn_ptr_atom, captured_atoms, original_fn_ty);

                ClosExpr::Let(name.clone(), make_clos, Box::new(rest_clos))
            },
        }
    }

    fn compile_comp_with_env(
        comp: CompExpr,
        bound: &HashSet<Ident>,
        lambda_bound: &HashSet<Ident>,
        env_param: &str,
        env_field_index: &std::collections::HashMap<Ident, usize>,
        fn_defs: &mut Vec<ClosFnDef>,
        gs: &mut Gensym,
    ) -> ClosCompExpr {
        match comp {
            CompExpr::Atom(a) => {
                let clos_a = compile_atom_with_env(a, bound, lambda_bound, env_param, env_field_index);
                ClosCompExpr::Atom(clos_a)
            },
            CompExpr::BinOp(op, l, r) => {
                let l_clos = compile_atom_with_env(l, bound, lambda_bound, env_param, env_field_index);
                let r_clos = compile_atom_with_env(r, bound, lambda_bound, env_param, env_field_index);
                ClosCompExpr::BinOp(op.clone(), l_clos, r_clos)
            },
            CompExpr::UnaryOp(op, a) => {
                let a_clos = compile_atom_with_env(a, bound, lambda_bound, env_param, env_field_index);
                ClosCompExpr::UnaryOp(op.clone(), a_clos)
            },
            CompExpr::App(fn_a, args) => {
                let fn_clos = compile_atom_with_env(fn_a, bound, lambda_bound, env_param, env_field_index);
                let args_clos: Vec<AExpr> = args
                    .iter()
                    .map(|a| compile_atom_with_env((*a).clone(), bound, lambda_bound, env_param, env_field_index))
                    .collect();
                ClosCompExpr::App(fn_clos, args_clos)
            },
            CompExpr::If(cond, thn, els) => {
                let cond_clos = compile_atom_with_env(cond, bound, lambda_bound, env_param, env_field_index);
                let thn_clos = compile_anf_with_env(*thn, bound, lambda_bound, env_param, env_field_index, fn_defs, gs);
                let els_clos = compile_anf_with_env(*els, bound, lambda_bound, env_param, env_field_index, fn_defs, gs);
                ClosCompExpr::If(cond_clos, Box::new(thn_clos), Box::new(els_clos))
            },
            CompExpr::Lambda(params, ret_ty, body) => {
                let fn_name = gs.fresh_with_prefix("lambda$");
                let nested_env_param = gs.fresh_with_prefix("env$");

                let mut nested_lambda_bound: HashSet<Ident> = HashSet::new();
                for (pname, _) in params.clone() {
                    nested_lambda_bound.insert(pname);
                }
                nested_lambda_bound.insert(nested_env_param.clone());

                let (conv_body, fv_nested) =
                    convert_closure_body(*body, bound, &nested_lambda_bound, &nested_env_param, fn_defs, gs);

                let fn_def = ClosFnDef {
                    name: fn_name.clone(),
                    env_param: nested_env_param,
                    params: params.clone(),
                    return_type: ret_ty.clone(),
                    body: conv_body,
                };
                fn_defs.push(fn_def);

                let fn_ptr_atom =
                    AExpr::Var(fn_name.clone(), Type::Arrow(Box::new(Type::Unit), Box::new(ret_ty.clone())));

                let mut captured_atoms: Vec<AExpr> = Vec::new();
                for (fv_name, fv_ty) in &fv_nested {
                    let atom = compile_var_with_env(fv_name, fv_ty, bound, lambda_bound, env_param, env_field_index);
                    captured_atoms.push(atom);
                }

                let original_fn_ty = params
                    .iter()
                    .rfold(ret_ty.clone(), |acc, (_, pty)| Type::Arrow(Box::new(pty.clone()), Box::new(acc)));

                ClosCompExpr::MakeClosure(fn_ptr_atom, captured_atoms, original_fn_ty)
            },
        }
    }

    fn compile_atom_with_env(
        a: AExpr,
        _bound: &HashSet<Ident>,
        _lambda_bound: &HashSet<Ident>,
        _env_param: &str,
        _env_field_index: &std::collections::HashMap<Ident, usize>,
    ) -> AExpr {
        a
    }

    fn compile_var_with_env(
        name: &str,
        ty: &Type,
        bound: &HashSet<Ident>,
        _lambda_bound: &HashSet<Ident>,
        _env_param: &str,
        env_field_index: &std::collections::HashMap<Ident, usize>,
    ) -> AExpr {
        if env_field_index.contains_key(name) {
            AExpr::Var(name.to_string(), ty.clone())
        } else if bound.contains(name) {
            AExpr::Var(name.to_string(), ty.clone())
        } else {
            AExpr::Var(name.to_string(), Type::Unit)
        }
    }

    let body_converted =
        compile_anf_with_env(body, outer_bound, lambda_bound, env_param_name, &env_field_index, fn_defs, gs);

    (body_converted, free_vars)
}

fn compile_anf(anf: AnfExpr, bound: &HashSet<Ident>, fn_defs: &mut Vec<ClosFnDef>, gs: &mut Gensym) -> ClosExpr {
    match anf {
        AnfExpr::Complex(comp) => ClosExpr::Complex(compile_comp(comp, bound, fn_defs, gs)),
        AnfExpr::Let(name, rhs, body) => {
            let rhs_clos = compile_comp(rhs, bound, fn_defs, gs);
            let mut new_bound = bound.clone();
            new_bound.insert(name.clone());
            let body_clos = compile_anf(*body, &new_bound, fn_defs, gs);
            ClosExpr::Let(name.clone(), rhs_clos, Box::new(body_clos))
        },
        AnfExpr::LetRec(name, params, ret_ty, body, rest) => {
            let fn_inner_name = gs.fresh_with_prefix("recur$");
            let env_param_name = gs.fresh_with_prefix("env$");

            let mut rec_bound: HashSet<Ident> = HashSet::new();
            rec_bound.insert(name.clone());
            for (pname, _) in params.clone() {
                rec_bound.insert(pname);
            }
            rec_bound.insert(env_param_name.clone());

            let (conv_body, free_vars) = convert_closure_body(*body, bound, &rec_bound, &env_param_name, fn_defs, gs);

            let fn_def = ClosFnDef {
                name: fn_inner_name.clone(),
                env_param: env_param_name,
                params: params.clone(),
                return_type: ret_ty.clone(),
                body: conv_body,
            };
            fn_defs.push(fn_def);

            let fn_ptr_atom =
                AExpr::Var(fn_inner_name.clone(), Type::Arrow(Box::new(Type::Unit), Box::new(ret_ty.clone())));

            let mut captured_atoms: Vec<AExpr> = Vec::new();
            for (fv_name, fv_ty) in &free_vars {
                let atom = if bound.contains(fv_name) {
                    AExpr::Var(fv_name.clone(), fv_ty.clone())
                } else {
                    AExpr::Var(fv_name.clone(), fv_ty.clone())
                };
                captured_atoms.push(atom);
            }

            let original_fn_ty = params
                .iter()
                .rfold(ret_ty.clone(), |acc, (_, pty)| Type::Arrow(Box::new(pty.clone()), Box::new(acc)));

            let mut rest_bound = bound.clone();
            rest_bound.insert(name.clone());
            let rest_clos = compile_anf(*rest, &rest_bound, fn_defs, gs);

            let make_clos = ClosCompExpr::MakeClosure(fn_ptr_atom, captured_atoms, original_fn_ty);

            ClosExpr::Let(name.clone(), make_clos, Box::new(rest_clos))
        },
    }
}

pub fn closure_convert(anf: AnfExpr) -> ClosProgram {
    let mut fn_defs = Vec::new();
    let mut gs = Gensym::new();
    let bound = HashSet::new();
    let body = compile_anf(anf, &bound, &mut fn_defs, &mut gs);
    ClosProgram { fn_defs, body }
}

pub fn merge_clos_programs(mut programs: Vec<ClosProgram>) -> ClosProgram {
    if programs.is_empty() {
        return ClosProgram { fn_defs: Vec::new(), body: ClosExpr::Complex(ClosCompExpr::Atom(AExpr::Unit)) };
    }
    if programs.len() == 1 {
        return programs.remove(0);
    }
    let mut all_fn_defs = Vec::new();
    let mut first = programs.remove(0);
    all_fn_defs.append(&mut first.fn_defs);
    for mut prog in programs {
        all_fn_defs.append(&mut prog.fn_defs);
    }
    ClosProgram { fn_defs: all_fn_defs, body: first.body }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{BinOp, Type};

    fn v(name: &str, ty: Type) -> AExpr {
        AExpr::Var(name.to_string(), ty)
    }

    fn int_a(n: i64) -> AExpr {
        AExpr::Int(n)
    }

    fn bool_a(b: bool) -> AExpr {
        AExpr::Bool(b)
    }

    #[test]
    fn test_simple_lambda_no_free_vars() {
        // fun (x : Int) : Int => x + 1
        let anf = AnfExpr::Complex(CompExpr::Lambda(
            vec![("x".to_string(), Type::Int)],
            Type::Int,
            Box::new(AnfExpr::Complex(CompExpr::BinOp(
                BinOp::Add,
                AExpr::Var("x".to_string(), Type::Int),
                AExpr::Int(1),
            ))),
        ));

        let prog = closure_convert(anf);
        assert_eq!(prog.fn_defs.len(), 1);
        let fn_def = &prog.fn_defs[0];
        assert_eq!(fn_def.params.len(), 1);
        assert_eq!(fn_def.params[0].0, "x");
        assert_eq!(fn_def.return_type, Type::Int);

        match &prog.body {
            ClosExpr::Complex(ClosCompExpr::MakeClosure(_, captured, _)) => {
                assert_eq!(captured.len(), 0);
            },
            _ => panic!("Expected MakeClosure in body"),
        }
    }

    #[test]
    fn test_lambda_with_free_var() {
        // let y = 5 in fun (x : Int) : Int => x + y
        let anf = AnfExpr::Let(
            "y".to_string(),
            CompExpr::Atom(AExpr::Int(5)),
            Box::new(AnfExpr::Complex(CompExpr::Lambda(
                vec![("x".to_string(), Type::Int)],
                Type::Int,
                Box::new(AnfExpr::Complex(CompExpr::BinOp(
                    BinOp::Add,
                    AExpr::Var("x".to_string(), Type::Int),
                    AExpr::Var("y".to_string(), Type::Int),
                ))),
            ))),
        );

        let prog = closure_convert(anf);
        assert_eq!(prog.fn_defs.len(), 1);

        match &prog.body {
            ClosExpr::Let(name, ClosCompExpr::Atom(AExpr::Int(5)), body) => {
                assert_eq!(name, "y");
                match body.as_ref() {
                    ClosExpr::Complex(ClosCompExpr::MakeClosure(_, captured, _)) => {
                        assert_eq!(captured.len(), 1);
                        assert_eq!(captured[0], AExpr::Var("y".to_string(), Type::Int));
                    },
                    _ => panic!("Expected MakeClosure"),
                }
            },
            _ => panic!("Expected Let structure"),
        }
    }

    #[test]
    fn test_simple_arithmetic_no_lambdas() {
        // 1 + 2
        let anf = AnfExpr::Complex(CompExpr::BinOp(BinOp::Add, int_a(1), int_a(2)));

        let prog = closure_convert(anf);
        assert_eq!(prog.fn_defs.len(), 0);
        assert_eq!(prog.body, ClosExpr::Complex(ClosCompExpr::BinOp(BinOp::Add, int_a(1), int_a(2),)));
    }

    #[test]
    fn test_if_with_lambda_in_branch() {
        // if true then (fun (x: Int): Int => x) else (fun (x: Int): Int => x + 1)
        let lambda1 = CompExpr::Lambda(
            vec![("x".to_string(), Type::Int)],
            Type::Int,
            Box::new(AnfExpr::Complex(CompExpr::Atom(AExpr::Var("x".to_string(), Type::Int)))),
        );
        let lambda2 = CompExpr::Lambda(
            vec![("x".to_string(), Type::Int)],
            Type::Int,
            Box::new(AnfExpr::Complex(CompExpr::BinOp(
                BinOp::Add,
                AExpr::Var("x".to_string(), Type::Int),
                AExpr::Int(1),
            ))),
        );
        let anf = AnfExpr::Complex(CompExpr::If(
            AExpr::Bool(true),
            Box::new(AnfExpr::Complex(lambda1)),
            Box::new(AnfExpr::Complex(lambda2)),
        ));

        let prog = closure_convert(anf);
        assert_eq!(prog.fn_defs.len(), 2);
        match &prog.body {
            ClosExpr::Complex(ClosCompExpr::If(_, thn, els)) => {
                match thn.as_ref() {
                    ClosExpr::Complex(ClosCompExpr::MakeClosure(_, captured, _)) => {
                        assert_eq!(captured.len(), 0);
                    },
                    _ => panic!("Expected MakeClosure in then"),
                }
                match els.as_ref() {
                    ClosExpr::Complex(ClosCompExpr::MakeClosure(_, captured, _)) => {
                        assert_eq!(captured.len(), 0);
                    },
                    _ => panic!("Expected MakeClosure in else"),
                }
            },
            _ => panic!("Expected If"),
        }
    }

    #[test]
    fn test_letrec_no_free_vars() {
        // let rec f (x : Int) : Int = x in f 1
        let anf = AnfExpr::LetRec(
            "f".to_string(),
            vec![("x".to_string(), Type::Int)],
            Type::Int,
            Box::new(AnfExpr::Complex(CompExpr::Atom(AExpr::Var("x".to_string(), Type::Int)))),
            Box::new(AnfExpr::Complex(CompExpr::App(
                AExpr::Var("f".to_string(), Type::Arrow(Box::new(Type::Int), Box::new(Type::Int))),
                vec![AExpr::Int(1)],
            ))),
        );

        let prog = closure_convert(anf);
        assert_eq!(prog.fn_defs.len(), 1);

        match &prog.body {
            ClosExpr::Let(name, _, _) => {
                assert_eq!(name, "f");
            },
            _ => panic!("Expected Let binding for f"),
        }
    }

    #[test]
    fn test_free_vars_computation() {
        // Lambda body: x + y where y is free
        let anf = AnfExpr::Complex(CompExpr::Lambda(
            vec![("x".to_string(), Type::Int)],
            Type::Int,
            Box::new(AnfExpr::Complex(CompExpr::BinOp(
                BinOp::Add,
                AExpr::Var("x".to_string(), Type::Int),
                AExpr::Var("y".to_string(), Type::Int),
            ))),
        ));

        let mut bound = HashSet::new();
        let mut free = HashSet::new();
        free_vars_anf(&anf, &mut bound, &mut free);

        assert_eq!(free.len(), 1);
        assert!(free.contains("y"));
    }
}
