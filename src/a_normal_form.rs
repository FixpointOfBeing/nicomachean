use crate::gensym::Gensym;
use crate::syntax::{BinOp, Ident, Type, UnaryOp};
use crate::typechecker::TypedExpr;

#[derive(Debug, Clone, PartialEq)]
pub enum AExpr {
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    Var(Ident, Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompExpr {
    Atom(AExpr),
    BinOp(BinOp, AExpr, AExpr),
    UnaryOp(UnaryOp, AExpr),
    App(AExpr, Vec<AExpr>),
    If(AExpr, Box<AnfExpr>, Box<AnfExpr>),
    Lambda(Vec<(Ident, Type)>, Type, Box<AnfExpr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnfExpr {
    Complex(CompExpr),
    Let(Ident, CompExpr, Box<AnfExpr>),
    LetRec(Ident, Vec<(Ident, Type)>, Type, Box<AnfExpr>, Box<AnfExpr>),
}

enum Binding {
    Let(Ident, CompExpr),
    LetRec(Ident, Vec<(Ident, Type)>, Type, AnfExpr),
}
type Bindings = Vec<Binding>;

fn expr_type(expr: &TypedExpr) -> Type {
    match expr {
        TypedExpr::Unit => Type::Unit,
        TypedExpr::Bool(_) => Type::Bool,
        TypedExpr::Int(_) => Type::Int,
        TypedExpr::Float(_) => Type::Float,
        TypedExpr::Var(_, ty) => ty.clone(),
        TypedExpr::BinOp(_, _, _, ty) => ty.clone(),
        TypedExpr::UnaryOp(_, _, ty) => ty.clone(),
        TypedExpr::Ann(_, ty) => ty.clone(),
        TypedExpr::If(_, _, _, ty) => ty.clone(),
        TypedExpr::Let(_, _, _, _, ty) => ty.clone(),
        TypedExpr::LetRec(_, _, _, _, _, ty) => ty.clone(),
        TypedExpr::App(_, _, ty) => ty.clone(),
        TypedExpr::Lambda(_, _, _, ty) => ty.clone(),
    }
}

fn to_atom(expr: TypedExpr, gs: &mut Gensym, bindings: &mut Bindings) -> AExpr {
    match expr {
        TypedExpr::Unit => AExpr::Unit,
        TypedExpr::Bool(b) => AExpr::Bool(b),
        TypedExpr::Int(i) => AExpr::Int(i),
        TypedExpr::Float(f) => AExpr::Float(f),
        TypedExpr::Var(name, ty) => AExpr::Var(name, ty),
        _ => {
            let ty = expr_type(&expr);

            let c = collect_bindings(expr, gs, bindings);
            if let CompExpr::Atom(a) = c {
                return a;
            }
            let name = gs.fresh();
            bindings.push(Binding::Let(name.clone(), c));
            AExpr::Var(name, ty)
        },
    }
}

fn collect_bindings(expr: TypedExpr, gs: &mut Gensym, bindings: &mut Bindings) -> CompExpr {
    match expr {
        TypedExpr::Unit => CompExpr::Atom(AExpr::Unit),
        TypedExpr::Bool(b) => CompExpr::Atom(AExpr::Bool(b)),
        TypedExpr::Int(i) => CompExpr::Atom(AExpr::Int(i)),
        TypedExpr::Float(f) => CompExpr::Atom(AExpr::Float(f)),
        TypedExpr::Var(name, ty) => CompExpr::Atom(AExpr::Var(name, ty)),
        TypedExpr::BinOp(op, left, right, _) => {
            let left_atom = to_atom(*left, gs, bindings);
            let right_atom = to_atom(*right, gs, bindings);
            CompExpr::BinOp(op, left_atom, right_atom)
        },
        TypedExpr::UnaryOp(op, operand, _) => {
            let operand_atom = to_atom(*operand, gs, bindings);
            CompExpr::UnaryOp(op, operand_atom)
        },
        TypedExpr::Ann(inner, _) => collect_bindings(*inner, gs, bindings),
        TypedExpr::If(cond, thn, els, _) => {
            let cond_atom = to_atom(*cond, gs, bindings);
            let then_anf = normalize(*thn, gs);
            let else_anf = normalize(*els, gs);
            CompExpr::If(cond_atom, Box::new(then_anf), Box::new(else_anf))
        },
        TypedExpr::App(func, args, _) => {
            let func_atom = to_atom(*func, gs, bindings);
            let mut args_atom = Vec::with_capacity(args.len());
            for arg in args {
                let arg_atom = to_atom(arg, gs, bindings);
                args_atom.push(arg_atom);
            }
            CompExpr::App(func_atom, args_atom)
        },
        TypedExpr::Lambda(params, ret_ty, body, _) => {
            let body_anf = normalize(*body, gs);
            CompExpr::Lambda(params, ret_ty, Box::new(body_anf))
        },
        TypedExpr::Let(name, _, rhs, body, _) => {
            let rhs_comp = collect_bindings(*rhs, gs, bindings);
            bindings.push(Binding::Let(name, rhs_comp));
            collect_bindings(*body, gs, bindings)
        },
        TypedExpr::LetRec(fname, fparams, fty, fbody, body, _) => {
            let fbody_anf = normalize(*fbody, gs);
            bindings.push(Binding::LetRec(fname, fparams, fty, fbody_anf));
            collect_bindings(*body, gs, bindings)
        },
    }
}

fn bindings_to_lets(bindings: Bindings, tail: AnfExpr) -> AnfExpr {
    bindings.into_iter().rev().fold(tail, |acc, b| match b {
        Binding::Let(name, c) => AnfExpr::Let(name, c, Box::new(acc)),
        Binding::LetRec(fname, fparams, fty, fbody) => {
            AnfExpr::LetRec(fname, fparams, fty, Box::new(fbody), Box::new(acc))
        },
    })
}

fn normalize(expr: TypedExpr, gs: &mut Gensym) -> AnfExpr {
    let mut bindings = Bindings::new();
    let c_expr = collect_bindings(expr, gs, &mut bindings);
    let tail = AnfExpr::Complex(c_expr);
    bindings_to_lets(bindings, tail)
}

pub fn anf_convert(expr: TypedExpr) -> AnfExpr {
    let mut gs = Gensym::new();
    normalize(expr, &mut gs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::BinOp;
    use crate::syntax::Type;

    fn v(name: &str, ty: Type) -> TypedExpr {
        TypedExpr::Var(name.to_string(), ty)
    }

    fn int(n: i64) -> TypedExpr {
        TypedExpr::Int(n)
    }

    fn bool(b: bool) -> TypedExpr {
        TypedExpr::Bool(b)
    }

    #[test]
    fn test_nested_binop() {
        // (1 + 2) * (3 + 4)
        // --->
        // let $0 = 1 + 2 in
        // let $1 = 3 + 4 in
        // $0 * $1
        let e = TypedExpr::BinOp(
            BinOp::Mul,
            Box::new(TypedExpr::BinOp(BinOp::Add, Box::new(int(1)), Box::new(int(2)), Type::Int)),
            Box::new(TypedExpr::BinOp(BinOp::Add, Box::new(int(3)), Box::new(int(4)), Type::Int)),
            Type::Int,
        );
        let anf = anf_convert(e);

        assert_eq!(
            anf,
            AnfExpr::Let(
                "$0".to_string(),
                CompExpr::BinOp(BinOp::Add, AExpr::Int(1), AExpr::Int(2)),
                Box::new(AnfExpr::Let(
                    "$1".to_string(),
                    CompExpr::BinOp(BinOp::Add, AExpr::Int(3), AExpr::Int(4)),
                    Box::new(AnfExpr::Complex(CompExpr::BinOp(
                        BinOp::Mul,
                        AExpr::Var("$0".to_string(), Type::Int),
                        AExpr::Var("$1".to_string(), Type::Int)
                    )))
                ))
            )
        );
    }

    #[test]
    fn test_atom_alone() {
        let e = int(5);
        let anf = anf_convert(e);
        assert_eq!(anf, AnfExpr::Complex(CompExpr::Atom(AExpr::Int(5))));
    }

    #[test]
    fn test_let_simple_no_extra_binding() {
        // let x = 1 + 2 in x
        // --->
        // let x = 1 + 2 in x
        let e = TypedExpr::Let(
            "x".to_string(),
            Type::Int,
            Box::new(TypedExpr::BinOp(BinOp::Add, Box::new(int(1)), Box::new(int(2)), Type::Int)),
            Box::new(v("x", Type::Int)),
            Type::Int,
        );

        let anf = anf_convert(e);
        assert_eq!(
            anf,
            AnfExpr::Let(
                "x".to_string(),
                CompExpr::BinOp(BinOp::Add, AExpr::Int(1), AExpr::Int(2)),
                Box::new(AnfExpr::Complex(CompExpr::Atom(AExpr::Var("x".to_string(), Type::Int))))
            )
        );
    }

    #[test]
    fn test_let_nested_inside_binop() {
        // 1 + (let x = 2 in x + 1)
        // --->
        // let x = 2 in
        // let $0 = x + 1 in
        // 1 + $0
        let e = TypedExpr::BinOp(
            BinOp::Add,
            Box::new(int(1)),
            Box::new(TypedExpr::Let(
                "x".to_string(),
                Type::Int,
                Box::new(int(2)),
                Box::new(TypedExpr::BinOp(BinOp::Add, Box::new(v("x", Type::Int)), Box::new(int(1)), Type::Int)),
                Type::Int,
            )),
            Type::Int,
        );

        let anf = anf_convert(e);
        assert_eq!(
            anf,
            AnfExpr::Let(
                "x".to_string(),
                CompExpr::Atom(AExpr::Int(2)),
                Box::new(AnfExpr::Let(
                    "$0".to_string(),
                    CompExpr::BinOp(BinOp::Add, AExpr::Var("x".to_string(), Type::Int), AExpr::Int(1)),
                    Box::new(AnfExpr::Complex(CompExpr::BinOp(
                        BinOp::Add,
                        AExpr::Int(1),
                        AExpr::Var("$0".to_string(), Type::Int)
                    )))
                ))
            )
        );
    }

    #[test]
    fn test_if_condition_gets_bound() {
        // if 1 < 2 then 1 else 2
        // --->
        // let $0 = 1 < 2
        // in if $0 then 1 else 2
        let e = TypedExpr::If(
            Box::new(TypedExpr::BinOp(BinOp::Lt, Box::new(int(1)), Box::new(int(2)), Type::Bool)),
            Box::new(int(1)),
            Box::new(int(2)),
            Type::Int,
        );

        let anf = anf_convert(e);
        assert_eq!(
            anf,
            AnfExpr::Let(
                "$0".to_string(),
                CompExpr::BinOp(BinOp::Lt, AExpr::Int(1), AExpr::Int(2)),
                Box::new(AnfExpr::Complex(CompExpr::If(
                    AExpr::Var("$0".to_string(), Type::Bool),
                    Box::new(AnfExpr::Complex(CompExpr::Atom(AExpr::Int(1)))),
                    Box::new(AnfExpr::Complex(CompExpr::Atom(AExpr::Int(2))))
                )))
            )
        );
    }

    #[test]
    fn test_if_branches_have_independent_bindings() {
        // if true
        // then 1 + (let x = 3 in x * x)
        // else (let x = 5 in x * 3) + 4
        // --->
        // if true
        // then let x = 3
        //      in let $0 = x * x
        //         in 1 + $0
        // else let x = 5
        //      in let $1 = x * 3
        //      in $1 + 4
        let e = TypedExpr::If(
            Box::new(bool(true)),
            Box::new(TypedExpr::BinOp(
                BinOp::Add,
                Box::new(int(1)),
                Box::new(TypedExpr::Let(
                    "x".to_string(),
                    Type::Int,
                    Box::new(int(3)),
                    Box::new(TypedExpr::BinOp(
                        BinOp::Mul,
                        Box::new(v("x", Type::Int)),
                        Box::new(v("x", Type::Int)),
                        Type::Int,
                    )),
                    Type::Int,
                )),
                Type::Int,
            )),
            Box::new(TypedExpr::BinOp(
                BinOp::Add,
                Box::new(TypedExpr::Let(
                    "x".to_string(),
                    Type::Int,
                    Box::new(int(5)),
                    Box::new(TypedExpr::BinOp(BinOp::Mul, Box::new(v("x", Type::Int)), Box::new(int(3)), Type::Int)),
                    Type::Int,
                )),
                Box::new(int(4)),
                Type::Int,
            )),
            Type::Int,
        );

        let anf = anf_convert(e);

        assert_eq!(
            anf,
            AnfExpr::Complex(CompExpr::If(
                AExpr::Bool(true),
                Box::new(AnfExpr::Let(
                    "x".to_string(),
                    CompExpr::Atom(AExpr::Int(3)),
                    Box::new(AnfExpr::Let(
                        "$0".to_string(),
                        CompExpr::BinOp(
                            BinOp::Mul,
                            AExpr::Var("x".to_string(), Type::Int),
                            AExpr::Var("x".to_string(), Type::Int)
                        ),
                        Box::new(AnfExpr::Complex(CompExpr::BinOp(
                            BinOp::Add,
                            AExpr::Int(1),
                            AExpr::Var("$0".to_string(), Type::Int)
                        )))
                    ))
                )),
                Box::new(AnfExpr::Let(
                    "x".to_string(),
                    CompExpr::Atom(AExpr::Int(5)),
                    Box::new(AnfExpr::Let(
                        "$1".to_string(),
                        CompExpr::BinOp(BinOp::Mul, AExpr::Var("x".to_string(), Type::Int), AExpr::Int(3)),
                        Box::new(AnfExpr::Complex(CompExpr::BinOp(
                            BinOp::Add,
                            AExpr::Var("$1".to_string(), Type::Int),
                            AExpr::Int(4)
                        )))
                    ))
                ))
            ))
        );
    }

    #[test]
    fn test_app_with_complex_arg() {
        // f (1 + 2) 3
        // --->
        // let $0 = 1 + 2
        // in f $0 3
        let fn_ty = Type::Arrow(Box::new(Type::Int), Box::new(Type::Arrow(Box::new(Type::Int), Box::new(Type::Int))));
        let e = TypedExpr::App(
            Box::new(v("f", fn_ty.clone().clone())),
            vec![
                TypedExpr::BinOp(BinOp::Add, Box::new(int(1)), Box::new(int(2)), Type::Int),
                int(3),
            ],
            Type::Int,
        );

        let anf = anf_convert(e);

        assert_eq!(
            anf,
            AnfExpr::Let(
                "$0".to_string(),
                CompExpr::BinOp(BinOp::Add, AExpr::Int(1), AExpr::Int(2)),
                Box::new(AnfExpr::Complex(CompExpr::App(
                    AExpr::Var("f".to_string(), fn_ty),
                    vec![AExpr::Var("$0".to_string(), Type::Int), AExpr::Int(3)]
                )))
            )
        );
    }

    #[test]
    fn test_lambda_body_normalized() {
        // fun (x : Int) : Int => x + 1
        // --->
        // fun (x : Int) : Int => x + 1
        let e = TypedExpr::Lambda(
            vec![("x".to_string(), Type::Int)],
            Type::Int,
            Box::new(TypedExpr::BinOp(BinOp::Add, Box::new(v("x", Type::Int)), Box::new(int(1)), Type::Int)),
            Type::Arrow(Box::new(Type::Int), Box::new(Type::Int)),
        );

        let anf = anf_convert(e);

        assert_eq!(
            anf,
            AnfExpr::Complex(CompExpr::Lambda(
                vec![("x".to_string(), Type::Int)],
                Type::Int,
                Box::new(AnfExpr::Complex(CompExpr::BinOp(
                    BinOp::Add,
                    AExpr::Var("x".to_string(), Type::Int),
                    AExpr::Int(1)
                )))
            ))
        );
    }

    #[test]
    fn test_letrec_and_call() {
        // let rec f (x : Int) : Int = x in f 1
        // --->
        // let rec f (x : Int) : Int = x in f 1
        let fn_ty = Type::Arrow(Box::new(Type::Int), Box::new(Type::Int));
        let e = TypedExpr::LetRec(
            "f".to_string(),
            vec![("x".to_string(), Type::Int)],
            Type::Int,
            Box::new(v("x", Type::Int)),
            Box::new(TypedExpr::App(Box::new(v("f", fn_ty.clone())), vec![int(1)], Type::Int)),
            Type::Int,
        );

        let anf = anf_convert(e);

        assert_eq!(
            anf,
            AnfExpr::LetRec(
                "f".to_string(),
                vec![("x".to_string(), Type::Int)],
                Type::Int,
                Box::new(AnfExpr::Complex(CompExpr::Atom(AExpr::Var("x".to_string(), Type::Int)))),
                Box::new(AnfExpr::Complex(CompExpr::App(AExpr::Var("f".to_string(), fn_ty), vec![AExpr::Int(1)])))
            )
        );
    }

    #[test]
    fn test_ann_wrapping_atom_no_extra_binding() {
        let e = TypedExpr::BinOp(BinOp::Add, Box::new(int(1)), Box::new(v("x", Type::Int)), Type::Int);

        let anf = anf_convert(e);

        assert_eq!(
            anf,
            AnfExpr::Complex(CompExpr::BinOp(BinOp::Add, AExpr::Int(1), AExpr::Var("x".to_string(), Type::Int)))
        );
    }

    #[test]
    fn test_let_body_is_var_no_extra_binding() {
        // 1 + (let y = 2 in y)
        // --->
        // let y = 2 in 1 + y
        let e = TypedExpr::BinOp(
            BinOp::Add,
            Box::new(int(1)),
            Box::new(TypedExpr::Let(
                "y".to_string(),
                Type::Int,
                Box::new(int(2)),
                Box::new(v("y", Type::Int)),
                Type::Int,
            )),
            Type::Int,
        );

        let anf = anf_convert(e);

        assert_eq!(
            anf,
            AnfExpr::Let(
                "y".to_string(),
                CompExpr::Atom(AExpr::Int(2)),
                Box::new(AnfExpr::Complex(CompExpr::BinOp(
                    BinOp::Add,
                    AExpr::Int(1),
                    AExpr::Var("y".to_string(), Type::Int)
                )))
            )
        );
    }
}
