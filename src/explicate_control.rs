use crate::{
    a_normal_form::AExpr,
    closure_conversion::{ClosCompExpr, ClosExpr},
    syntax::{BinOp, Ident, Type, UnaryOp},
};

#[derive(Debug, Clone, PartialEq)]
pub enum CAtom {
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    Var(Ident, Type),
}

// todo: CExpr应该携带Type信息
#[derive(Debug, Clone, PartialEq)]
pub enum CExpr {
    Atom(CAtom),
    BinOp(BinOp, CAtom, CAtom),
    UnaryOp(UnaryOp, CAtom),
    Call(CAtom, Vec<CAtom>),
    MakeClosure(CAtom, Vec<CAtom>, Type),
    Project(CAtom, usize, Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CStmt {
    Assign(Ident, CExpr, Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CTail {
    Return(CExpr),
    TailCall(CAtom, Vec<CAtom>),
    Seq(CStmt, Box<CTail>),
    If(CAtom, Box<CTail>, Box<CTail>),
}

pub fn explicate_assign(expr: ClosExpr, name: &str, cont: CTail) -> CTail {
    match expr {
        ClosExpr::Complex(cexpr) => match cexpr {
            ClosCompExpr::Atom(aexpr) => {
                let cexpr = CExpr::Atom(aexpr_to_catom(aexpr));
                let ty = type_of_cexpr(&cexpr);
                let stmt = CStmt::Assign(name.to_string(), cexpr, ty);
                CTail::Seq(stmt, Box::new(cont))
            },
            ClosCompExpr::BinOp(op, left, right) => {
                let cexpr = CExpr::BinOp(op, aexpr_to_catom(left), aexpr_to_catom(right));
                let ty = type_of_cexpr(&cexpr);
                let stmt = CStmt::Assign(name.to_string(), cexpr, ty);
                CTail::Seq(stmt, Box::new(cont))
            },
            ClosCompExpr::UnaryOp(op, operand) => {
                let cexpr = CExpr::UnaryOp(op, aexpr_to_catom(operand));
                let ty = type_of_cexpr(&cexpr);
                let stmt = CStmt::Assign(name.to_string(), cexpr, ty);
                CTail::Seq(stmt, Box::new(cont))
            },
            ClosCompExpr::App(func, args) => {
                let cexpr = CExpr::Call(aexpr_to_catom(func), args.into_iter().map(aexpr_to_catom).collect());
                let ty = type_of_cexpr(&cexpr);
                let stmt = CStmt::Assign(name.to_string(), cexpr, ty);
                CTail::Seq(stmt, Box::new(cont))
            },
            ClosCompExpr::If(cond, thn, els) => {
                let cond_catom = aexpr_to_catom(cond);
                let then_tail = explicate_assign(*thn, name, cont.clone());
                let else_tail = explicate_assign(*els, name, cont);
                CTail::If(cond_catom, Box::new(then_tail), Box::new(else_tail))
            },
            ClosCompExpr::MakeClosure(fn_ptr, captured, closure_type) => {
                let cexpr = CExpr::MakeClosure(
                    aexpr_to_catom(fn_ptr),
                    captured.into_iter().map(aexpr_to_catom).collect(),
                    closure_type,
                );
                let ty = type_of_cexpr(&cexpr);
                let stmt = CStmt::Assign(name.to_string(), cexpr, ty);
                CTail::Seq(stmt, Box::new(cont))
            },
            ClosCompExpr::Project(env, idx, field_type) => {
                let cexpr = CExpr::Project(aexpr_to_catom(env), idx, field_type);
                let ty = type_of_cexpr(&cexpr);
                let stmt = CStmt::Assign(name.to_string(), cexpr, ty);
                CTail::Seq(stmt, Box::new(cont))
            },
        },
        ClosExpr::Let(name1, cexpr, body) => {
            let inner_cont = explicate_assign(*body, name, cont);
            explicate_assign(ClosExpr::Complex(cexpr), &name1, inner_cont)
        },
    }
}

fn type_of_catom(atom: &CAtom) -> Type {
    match atom {
        CAtom::Unit => Type::Unit,
        CAtom::Bool(_) => Type::Bool,
        CAtom::Int(_) => Type::Int,
        CAtom::Float(_) => Type::Float,
        CAtom::Var(_, ty) => (*ty).clone(),
    }
}

fn type_of_cexpr(cexpr: &CExpr) -> Type {
    match cexpr {
        CExpr::Atom(catom) => type_of_catom(catom),
        CExpr::BinOp(op, left, _) => match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => type_of_catom(left),
            BinOp::And | BinOp::Or | BinOp::Eq | BinOp::Lt | BinOp::Gt | BinOp::Leq | BinOp::Geq => Type::Bool,
            BinOp::Neq => Type::Bool,
        },
        CExpr::UnaryOp(op, catom) => match op {
            UnaryOp::Neg => type_of_catom(catom),
            UnaryOp::Not => Type::Bool,
        },
        CExpr::Call(func, args) => {
            let mut curr_ty = type_of_catom(func);
            for _arg in args.iter() {
                match curr_ty {
                    Type::Arrow(_input_ty, output_ty) => {
                        curr_ty = *output_ty;
                    },
                    _ => unreachable!(),
                }
            }
            curr_ty
        },
        CExpr::MakeClosure(_, _, closure_type) => closure_type.clone(),
        CExpr::Project(_, _, field_type) => field_type.clone(),
    }
}

fn aexpr_to_catom(aexpr: AExpr) -> CAtom {
    match aexpr {
        AExpr::Unit => CAtom::Unit,
        AExpr::Bool(b) => CAtom::Bool(b),
        AExpr::Int(i) => CAtom::Int(i),
        AExpr::Float(f) => CAtom::Float(f),
        AExpr::Var(name, ty) => CAtom::Var(name, ty),
    }
}

fn explicate_tail(clos: ClosExpr) -> CTail {
    match clos {
        ClosExpr::Complex(cexpr) => match cexpr {
            ClosCompExpr::Atom(aexpr) => CTail::Return(CExpr::Atom(aexpr_to_catom(aexpr))),
            ClosCompExpr::BinOp(op, left, right) => {
                let cleft = aexpr_to_catom(left);
                let cright = aexpr_to_catom(right);
                let cexpr = CExpr::BinOp(op, cleft, cright);
                CTail::Return(cexpr)
            },
            ClosCompExpr::UnaryOp(op, aexpr) => {
                let catom = aexpr_to_catom(aexpr);
                let cexpr = CExpr::UnaryOp(op, catom);
                CTail::Return(cexpr)
            },
            ClosCompExpr::App(func, args) => {
                let func_catom = aexpr_to_catom(func);
                let args_catom = args.into_iter().map(aexpr_to_catom).collect();
                CTail::TailCall(func_catom, args_catom)
            },
            ClosCompExpr::If(cond, thn, els) => {
                let cond_catom = aexpr_to_catom(cond);
                let then_tail = explicate_tail(*thn);
                let else_tail = explicate_tail(*els);
                CTail::If(cond_catom, Box::new(then_tail), Box::new(else_tail))
            },
            ClosCompExpr::MakeClosure(fn_ptr, captured, closure_type) => {
                let cexpr = CExpr::MakeClosure(
                    aexpr_to_catom(fn_ptr),
                    captured.into_iter().map(aexpr_to_catom).collect(),
                    closure_type,
                );
                CTail::Return(cexpr)
            },
            ClosCompExpr::Project(env, idx, field_type) => {
                let cexpr = CExpr::Project(aexpr_to_catom(env), idx, field_type);
                CTail::Return(cexpr)
            },
        },
        ClosExpr::Let(name, rhs, body) => {
            let tail = explicate_tail(*body);
            explicate_assign(ClosExpr::Complex(rhs), &name, tail)
        },
    }
}

pub fn explicate_control_convert(clos: ClosExpr) -> CTail {
    explicate_tail(clos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::a_normal_form::AExpr;
    use crate::closure_conversion::{ClosCompExpr, ClosExpr};
    use crate::syntax::BinOp;
    use crate::syntax::UnaryOp;
    use rand::seq::IndexedRandom;

    fn anf_var(name: &str, ty: Type) -> AExpr {
        AExpr::Var(name.to_string(), ty)
    }

    fn random_type_with_depth(depth: usize) -> Type {
        let mut rng = rand::rng();

        if depth == 0 {
            let leaf_variants: &[fn() -> Type] = &[
                || Type::Unit,
                || Type::Bool,
                || Type::Float,
                || Type::Int,
                || Type::Var("$dummy".to_string()),
            ];
            return leaf_variants.choose(&mut rng).unwrap()();
        }

        let all_variants: &[fn(usize) -> Type] = &[
            |_| Type::Unit,
            |_| Type::Bool,
            |_| Type::Float,
            |_| Type::Int,
            |d| Type::Arrow(Box::new(random_type_with_depth(d - 1)), Box::new(random_type_with_depth(d - 1))),
            |_| Type::Var("$dummy".to_string()),
        ];
        all_variants.choose(&mut rng).unwrap()(depth)
    }

    fn random_type() -> Type {
        random_type_with_depth(5)
    }

    fn anf_int(n: i64) -> AExpr {
        AExpr::Int(n)
    }

    fn anf_bool(b: bool) -> AExpr {
        AExpr::Bool(b)
    }

    fn anf_atom_to_comp(a: AExpr) -> ClosCompExpr {
        ClosCompExpr::Atom(a)
    }

    fn anf_atom_to_anf(a: AExpr) -> ClosExpr {
        ClosExpr::Complex(ClosCompExpr::Atom(a))
    }

    fn anf_let(name: &str, rhs: ClosCompExpr, body: ClosExpr) -> ClosExpr {
        ClosExpr::Let(name.to_string(), rhs, Box::new(body))
    }

    fn c_var(name: &str, ty: Type) -> CAtom {
        CAtom::Var(name.to_string(), ty)
    }

    fn c_int(n: i64) -> CAtom {
        CAtom::Int(n)
    }

    fn c_bool(b: bool) -> CAtom {
        CAtom::Bool(b)
    }

    fn c_assign(name: &str, e: CExpr, cont: CTail) -> CTail {
        let ty = type_of_cexpr(&e);
        CTail::Seq(CStmt::Assign(name.to_string(), e, ty), Box::new(cont))
    }

    #[test]
    fn tail_atom_int() {
        let input = anf_atom_to_anf(anf_int(5));
        let expected = CTail::Return(CExpr::Atom(CAtom::Int(5)));
        assert_eq!(explicate_tail(input), expected);
    }

    #[test]
    fn tail_atom_var() {
        let ty = random_type();
        let input = anf_atom_to_anf(anf_var("x", ty.clone()));
        let expected = CTail::Return(CExpr::Atom(c_var("x", ty)));
        assert_eq!(explicate_tail(input), expected);
    }

    #[test]
    fn tail_binop() {
        let ty = Type::Int;
        let input = ClosExpr::Complex(ClosCompExpr::BinOp(BinOp::Add, anf_var("x", ty.clone()), anf_int(1)));
        let expected = CTail::Return(CExpr::BinOp(BinOp::Add, c_var("x", ty), c_int(1)));
        assert_eq!(explicate_tail(input), expected);
    }

    #[test]
    fn tail_unaryop() {
        let ty = Type::Float;
        let input = ClosExpr::Complex(ClosCompExpr::UnaryOp(UnaryOp::Neg, anf_var("x", ty.clone())));
        let expected = CTail::Return(CExpr::UnaryOp(UnaryOp::Neg, c_var("x", ty)));
        assert_eq!(explicate_tail(input), expected);
    }

    #[test]
    fn tail_app_becomes_tailcall() {
        let x_ty = random_type();
        let y_ty = random_type();
        let fn_ty =
            Type::Arrow(Box::new(x_ty.clone()), Box::new(Type::Arrow(Box::new(y_ty.clone()), Box::new(Type::Unit))));
        let input = ClosExpr::Complex(ClosCompExpr::App(
            anf_var("f", fn_ty.clone()),
            vec![anf_var("x", x_ty.clone()), anf_var("y", y_ty.clone())],
        ));
        let expected = CTail::TailCall(c_var("f", fn_ty), vec![c_var("x", x_ty), c_var("y", y_ty)]);
        assert_eq!(explicate_tail(input), expected);
    }

    #[test]
    fn tail_if_both_branches_are_tail() {
        let input = ClosExpr::Complex(ClosCompExpr::If(
            anf_var("c", Type::Bool),
            Box::new(anf_atom_to_anf(anf_int(1))),
            Box::new(anf_atom_to_anf(anf_int(2))),
        ));
        let expected = CTail::If(
            c_var("c", Type::Bool),
            Box::new(CTail::Return(CExpr::Atom(c_int(1)))),
            Box::new(CTail::Return(CExpr::Atom(c_int(2)))),
        );
        assert_eq!(explicate_tail(input), expected);
    }

    #[test]
    fn tail_let_single() {
        let input = anf_let("x", anf_atom_to_comp(anf_int(1)), anf_atom_to_anf(anf_var("x", Type::Int)));
        let expected = c_assign("x", CExpr::Atom(c_int(1)), CTail::Return(CExpr::Atom(c_var("x", Type::Int))));
        assert_eq!(explicate_tail(input), expected);
    }

    #[test]
    fn tail_let_nested_order_is_preserved() {
        let input = anf_let(
            "x",
            anf_atom_to_comp(anf_int(1)),
            anf_let(
                "y",
                ClosCompExpr::BinOp(BinOp::Add, anf_var("x", Type::Int), anf_int(1)),
                anf_atom_to_anf(anf_var("y", Type::Int)),
            ),
        );
        let expected = c_assign(
            "x",
            CExpr::Atom(c_int(1)),
            c_assign(
                "y",
                CExpr::BinOp(BinOp::Add, c_var("x", Type::Int), c_int(1)),
                CTail::Return(CExpr::Atom(c_var("y", Type::Int))),
            ),
        );
        assert_eq!(explicate_tail(input), expected);
    }

    #[test]
    fn tail_let_with_if_rhs() {
        let input = anf_let(
            "x",
            ClosCompExpr::If(
                anf_var("c", Type::Bool),
                Box::new(anf_atom_to_anf(anf_int(1))),
                Box::new(anf_atom_to_anf(anf_int(2))),
            ),
            anf_atom_to_anf(anf_var("x", Type::Int)),
        );
        let cont = CTail::Return(CExpr::Atom(c_var("x", Type::Int)));
        let expected = CTail::If(
            c_var("c", Type::Bool),
            Box::new(c_assign("x", CExpr::Atom(c_int(1)), cont.clone())),
            Box::new(c_assign("x", CExpr::Atom(c_int(2)), cont)),
        );
        assert_eq!(explicate_tail(input), expected);
    }

    #[test]
    fn assign_atom() {
        let input = anf_atom_to_anf(anf_int(5));
        let cont = CTail::Return(CExpr::Atom(c_var("x", Type::Int)));
        let expected = c_assign("x", CExpr::Atom(c_int(5)), cont.clone());
        assert_eq!(explicate_assign(input, "x", cont), expected);
    }

    #[test]
    fn assign_binop() {
        let input =
            ClosExpr::Complex(ClosCompExpr::BinOp(BinOp::Mul, anf_var("a", Type::Float), anf_var("b", Type::Float)));
        let cont = CTail::Return(CExpr::Atom(c_var("x", Type::Float)));
        let expected =
            c_assign("x", CExpr::BinOp(BinOp::Mul, c_var("a", Type::Float), c_var("b", Type::Float)), cont.clone());
        assert_eq!(explicate_assign(input, "x", cont), expected);
    }

    #[test]
    fn assign_unaryop() {
        let input = ClosExpr::Complex(ClosCompExpr::UnaryOp(UnaryOp::Not, anf_var("a", Type::Bool)));
        let cont = CTail::Return(CExpr::Atom(c_var("x", Type::Bool)));
        let expected = c_assign("x", CExpr::UnaryOp(UnaryOp::Not, c_var("a", Type::Bool)), cont.clone());
        assert_eq!(explicate_assign(input, "x", cont), expected);
    }

    #[test]
    fn assign_app_is_non_tail_call() {
        let a_ty = Type::Int;
        let b_ty = Type::Int;
        let result_ty = Type::Int;
        let f_ty = Type::Arrow(
            Box::new(a_ty.clone()),
            Box::new(Type::Arrow(Box::new(b_ty.clone()), Box::new(result_ty.clone()))),
        );
        let input = ClosExpr::Complex(ClosCompExpr::App(
            anf_var("f", f_ty.clone()),
            vec![anf_var("a", a_ty.clone()), anf_var("b", b_ty.clone())],
        ));
        let cont = CTail::Return(CExpr::Atom(c_var("x", result_ty.clone())));
        let expected =
            c_assign("x", CExpr::Call(c_var("f", f_ty), vec![c_var("a", a_ty), c_var("b", b_ty)]), cont.clone());
        assert_eq!(explicate_assign(input, "x", cont), expected);
    }

    #[test]
    fn assign_if_both_branches_assign_and_share_cont() {
        let input = ClosExpr::Complex(ClosCompExpr::If(
            anf_var("c", Type::Bool),
            Box::new(anf_atom_to_anf(anf_int(1))),
            Box::new(anf_atom_to_anf(anf_int(2))),
        ));
        let cont = CTail::Return(CExpr::Atom(c_var("x", Type::Int)));
        let expected = CTail::If(
            c_var("c", Type::Bool),
            Box::new(c_assign("x", CExpr::Atom(c_int(1)), cont.clone())),
            Box::new(c_assign("x", CExpr::Atom(c_int(2)), cont.clone())),
        );
        assert_eq!(explicate_assign(input, "x", cont.clone()), expected);
    }

    #[test]
    fn assign_if_nested_inside_let_rhs_and_outer_let() {
        let inner_let = anf_let(
            "x",
            ClosCompExpr::If(
                anf_var("c", Type::Bool),
                Box::new(anf_atom_to_anf(anf_int(1))),
                Box::new(anf_atom_to_anf(anf_int(2))),
            ),
            anf_atom_to_anf(anf_var("x", Type::Int)),
        );

        let final_cont = CTail::Return(CExpr::Atom(c_var("y", Type::Int)));
        let result = explicate_assign(inner_let, "y", final_cont.clone());

        let expected = CTail::If(
            c_var("c", Type::Bool),
            Box::new(c_assign(
                "x",
                CExpr::Atom(c_int(1)),
                c_assign("y", CExpr::Atom(c_var("x", Type::Int)), final_cont.clone()),
            )),
            Box::new(c_assign(
                "x",
                CExpr::Atom(c_int(2)),
                c_assign("y", CExpr::Atom(c_var("x", Type::Int)), final_cont),
            )),
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn assign_let_forwards_correctly() {
        let input = anf_let("a", anf_atom_to_comp(anf_int(1)), anf_atom_to_anf(anf_var("a", Type::Int)));
        let cont = CTail::Return(CExpr::Atom(c_var("x", Type::Int)));
        let expected =
            c_assign("a", CExpr::Atom(c_int(1)), c_assign("x", CExpr::Atom(c_var("a", Type::Int)), cont.clone()));
        assert_eq!(explicate_assign(input, "x", cont), expected);
    }

    #[test]
    fn assign_true_false_atoms() {
        let input = anf_atom_to_anf(anf_bool(true));
        let cont = CTail::Return(CExpr::Atom(c_var("x", Type::Bool)));
        let expected = c_assign("x", CExpr::Atom(c_bool(true)), cont.clone());
        assert_eq!(explicate_assign(input, "x", cont), expected);
    }
}
