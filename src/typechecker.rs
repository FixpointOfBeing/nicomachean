// use llvm_ir::types::Typed;

use crate::syntax::{BinOp, Expr, Ident, Type, UnaryOp};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TypedExpr {
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    Var(Ident, Type),
    BinOp(BinOp, Box<TypedExpr>, Box<TypedExpr>, Type),
    UnaryOp(UnaryOp, Box<TypedExpr>, Type),
    Ann(Box<TypedExpr>, Type),
    If(Box<TypedExpr>, Box<TypedExpr>, Box<TypedExpr>, Type),
    Let(Ident, Type, Box<TypedExpr>, Box<TypedExpr>, Type),
    LetRec(
        Ident,              // function name
        Vec<(Ident, Type)>, // function parameters with their types
        Type,               // function return type
        Box<TypedExpr>,     // function body
        Box<TypedExpr>,     // expression after the let rec
        Type,
    ),
    App(Box<TypedExpr>, Vec<TypedExpr>, Type),
    Lambda(Vec<(Ident, Type)>, Type, Box<TypedExpr>, Type),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeError {
    UnboundVariable(Ident),
    Mismatch { expected: Type, found: Type },
    ReturnTypeMismatch { expected: Type, found: Type },
    NotAFunction(Type),
    ArityMismatch { expected: usize, found: usize },
    BranchMismatch { then_ty: Type, else_ty: Type },
    InvalidOperands { op: String, left: Type, right: Type },
    InvalidUnary { op: String, ty: Type },
    AnnotationMismatch { annotated: Type, inferred: Type },
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeError::UnboundVariable(name) => {
                write!(f, "Unbound variable: {}", name)
            },
            TypeError::Mismatch { expected, found } => {
                write!(f, "Type mismatch: expected {:?}, found {:?}", expected, found)
            },
            TypeError::ReturnTypeMismatch { expected, found } => {
                write!(f, "Return type mismatch: expected {:?}, found {:?}", expected, found)
            },
            TypeError::NotAFunction(ty) => {
                write!(f, "Not a function: {:?}", ty)
            },
            TypeError::ArityMismatch { expected, found } => {
                write!(f, "Arity mismatch: expected {} args, got {}", expected, found)
            },
            TypeError::BranchMismatch { then_ty, else_ty } => {
                write!(f, "If branches have different types: then={:?}, else={:?}", then_ty, else_ty)
            },
            TypeError::InvalidOperands { op, left, right } => {
                write!(f, "Operator `{}` cannot be applied to {:?} and {:?}", op, left, right)
            },
            TypeError::InvalidUnary { op, ty } => {
                write!(f, "Operator `{}` cannot be applied to {:?}", op, ty)
            },
            TypeError::AnnotationMismatch { annotated, inferred } => {
                write!(f, "Annotation mismatch: declared {:?}, inferred {:?}", annotated, inferred)
            },
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Context(HashMap<Ident, Type>);

impl Context {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.0.get(name)
    }

    pub fn extend(&self, name: Ident, ty: Type) -> Self {
        let mut inner = self.0.clone();
        inner.insert(name, ty);
        Self(inner)
    }
}

pub fn typecheck(expr: Expr) -> Result<(Type, TypedExpr), TypeError> {
    infer(&Context::new(), expr)
}

pub fn typecheck_with_ctx(ctx: &Context, expr: Expr) -> Result<(Type, TypedExpr), TypeError> {
    infer(ctx, expr)
}

fn infer(ctx: &Context, expr: Expr) -> Result<(Type, TypedExpr), TypeError> {
    match expr {
        Expr::Unit => Ok((Type::Unit, TypedExpr::Unit)),
        Expr::Bool(b) => Ok((Type::Bool, TypedExpr::Bool(b))),
        Expr::Int(i) => Ok((Type::Int, TypedExpr::Int(i))),
        Expr::Float(f) => Ok((Type::Float, TypedExpr::Float(f))),

        Expr::Var(name) => match ctx.lookup(&name) {
            Some(ty) => Ok(((*ty).clone(), TypedExpr::Var(name.to_string(), (*ty).clone()))),
            None => Err(TypeError::UnboundVariable(name.clone())),
        },

        Expr::Ann(inner, ann_ty) => {
            let res = infer(ctx, *inner)?;
            let inferred = res.0.clone();
            if inferred != ann_ty {
                return Err(TypeError::AnnotationMismatch { annotated: ann_ty, inferred });
            }
            Ok(res)
        },

        Expr::UnaryOp(op, operand) => {
            let (ty, typed_operand) = infer(ctx, *operand)?;
            match op {
                UnaryOp::Neg => match &ty {
                    Type::Int | Type::Float => Ok((ty.clone(), (TypedExpr::UnaryOp(op, Box::new(typed_operand), ty)))),
                    _ => Err(TypeError::InvalidUnary { op: "-".to_string(), ty }),
                },
                UnaryOp::Not => match &ty {
                    Type::Bool => Ok((ty.clone(), (TypedExpr::UnaryOp(op, Box::new(typed_operand), ty)))),
                    _ => Err(TypeError::InvalidUnary { op: "!".to_string(), ty }),
                },
            }
        },

        Expr::BinOp(op, lhs, rhs) => {
            let (left_ty, left_ty_e) = infer(ctx, *lhs)?;
            let (right_ty, right_ty_e) = infer(ctx, *rhs)?;
            infer_binop(op, left_ty, left_ty_e, right_ty, right_ty_e)
        },

        Expr::If(cond, thn, els) => {
            let (cond_ty, typed_cond_expr) = infer(ctx, *cond)?;
            check(&Type::Bool, &cond_ty)?;

            let (then_ty, typed_then_expr) = infer(ctx, *thn)?;
            let (else_ty, typed_else_expr) = infer(ctx, *els)?;

            if then_ty != else_ty {
                return Err(TypeError::BranchMismatch { then_ty, else_ty });
            }
            Ok((
                then_ty.clone(),
                TypedExpr::If(Box::new(typed_cond_expr), Box::new(typed_then_expr), Box::new(typed_else_expr), then_ty),
            ))
        },

        Expr::Let(name, ann, rhs, body) => {
            let (rhs_ty, typed_rhs) = infer(ctx, *rhs)?;

            if let Some(ann_ty) = ann {
                if rhs_ty != ann_ty {
                    return Err(TypeError::AnnotationMismatch { annotated: ann_ty, inferred: rhs_ty });
                }
            }

            let ctx2 = ctx.extend(name.clone(), rhs_ty.clone());
            let (body_ty, typed_body) = infer(&ctx2, *body)?;
            Ok((body_ty.clone(), TypedExpr::Let(name, rhs_ty, Box::new(typed_rhs), Box::new(typed_body), body_ty)))
        },

        Expr::LetRec(fname, fparams, fret_ty, body, rest) => {
            let fn_ty = build_arrow(fparams.iter().map(|(_, t)| t.clone()).collect(), fret_ty.clone());

            let mut body_ctx = ctx.extend(fname.clone(), fn_ty.clone());
            for (param_name, param_ty) in fparams.clone() {
                body_ctx = body_ctx.extend(param_name, param_ty);
            }

            let (fbody_ty, typed_fbody) = infer(&body_ctx, *body)?;
            if fbody_ty != fret_ty {
                return Err(TypeError::AnnotationMismatch { annotated: fret_ty, inferred: fbody_ty });
            }

            let rest_ctx = ctx.extend(fname.clone(), fn_ty);
            let (body_ty, typed_body) = infer(&rest_ctx, *rest)?;
            let typed_letrec = TypedExpr::LetRec(
                fname,
                fparams,
                fret_ty,
                Box::new(typed_fbody),
                Box::new(typed_body),
                body_ty.clone(),
            );
            Ok((body_ty, typed_letrec))
        },

        Expr::Lambda(params, opty, body) => {
            let mut lam_ctx = ctx.clone();
            let mut param_tys = Vec::new();
            for (pname, pty) in params.clone() {
                lam_ctx = lam_ctx.extend(pname, pty.clone());
                param_tys.push(pty);
            }
            let (body_ty, typed_body) = infer(&lam_ctx, *body)?;
            if let Some(rt_ty) = opty {
                if body_ty != rt_ty {
                    return Err(TypeError::ReturnTypeMismatch { expected: rt_ty, found: body_ty });
                }
            }
            let lambda_ty = build_arrow(param_tys, body_ty.clone());
            let typed_lambda = TypedExpr::Lambda(params, body_ty, Box::new(typed_body), lambda_ty.clone());
            Ok((lambda_ty, typed_lambda))
        },

        Expr::App(func, args) => {
            let (fn_ty, typed_fn) = infer(ctx, *func)?;
            let mut typed_args = vec![];
            let mut curr_ty = fn_ty;
            for arg in args {
                match curr_ty {
                    Type::Arrow(param_ty, ret_ty) => {
                        let (arg_ty, typed_arg) = infer(ctx, arg)?;
                        if arg_ty != *param_ty {
                            return Err(TypeError::Mismatch { expected: *param_ty, found: arg_ty });
                        }
                        typed_args.push(typed_arg);
                        curr_ty = *ret_ty;
                    },
                    other => {
                        return Err(TypeError::NotAFunction(other));
                    },
                }
            }
            Ok((curr_ty.clone(), TypedExpr::App(Box::new(typed_fn), typed_args, curr_ty)))
        },
    }
}

fn check(expected: &Type, inferred: &Type) -> Result<(), TypeError> {
    if inferred != expected {
        Err(TypeError::Mismatch { expected: expected.clone(), found: inferred.clone() })
    } else {
        Ok(())
    }
}

fn build_arrow(params: Vec<Type>, ret: Type) -> Type {
    params
        .into_iter()
        .rev()
        .fold(ret, |acc, p| Type::Arrow(Box::new(p), Box::new(acc)))
}

fn infer_binop(
    op: BinOp,
    left_ty: Type,
    left_ty_e: TypedExpr,
    right_ty: Type,
    right_ty_e: TypedExpr,
) -> Result<(Type, TypedExpr), TypeError> {
    let op_str = format!("{:?}", op);
    let make_ty_expr = |ty: Type| TypedExpr::BinOp(op.clone(), Box::new(left_ty_e), Box::new(right_ty_e), ty);
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => match (&left_ty, &right_ty) {
            (Type::Int, Type::Int) => Ok((Type::Int, make_ty_expr(Type::Int))),
            (Type::Float, Type::Float) => Ok((Type::Float, make_ty_expr(Type::Float))),
            _ => Err(TypeError::InvalidOperands { op: op_str, left: left_ty, right: right_ty }),
        },

        BinOp::Lt | BinOp::Gt | BinOp::Leq | BinOp::Geq => match (&left_ty, &right_ty) {
            (Type::Int, Type::Int) | (Type::Float, Type::Float) => Ok((Type::Bool, make_ty_expr(Type::Bool))),
            _ => Err(TypeError::InvalidOperands { op: op_str, left: left_ty, right: right_ty }),
        },

        BinOp::Eq | BinOp::Neq => {
            if left_ty == right_ty {
                Ok((Type::Bool, make_ty_expr(Type::Bool)))
            } else {
                Err(TypeError::InvalidOperands { op: op_str, left: left_ty, right: right_ty })
            }
        },

        BinOp::And | BinOp::Or => match (&left_ty, &right_ty) {
            (Type::Bool, Type::Bool) => Ok((Type::Bool, make_ty_expr(Type::Bool))),
            _ => Err(TypeError::InvalidOperands { op: op_str, left: left_ty, right: right_ty }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::Type;
    use lalrpop_util::lalrpop_mod;
    lalrpop_mod!(pub parser);

    fn tc(src: &str) -> Result<Type, TypeError> {
        let expr = parser::ExprParser::new()
            .parse(src)
            .unwrap_or_else(|e| panic!("failed to parse `{}`: {:?}", src, e));
        match typecheck(*expr) {
            Ok((ty, _)) => Ok(ty),
            Err(e) => Err(e),
        }
    }

    #[test]
    fn test_unit() {
        assert_eq!(tc("()"), Ok(Type::Unit));
    }

    #[test]
    fn test_bool() {
        assert_eq!(tc("true"), Ok(Type::Bool));
    }

    #[test]
    fn test_int() {
        assert_eq!(tc("42"), Ok(Type::Int));
    }

    #[test]
    fn test_float() {
        assert_eq!(tc("3.14"), Ok(Type::Float));
    }

    #[test]
    fn test_neg_int() {
        assert_eq!(tc("-1"), Ok(Type::Int));
    }

    #[test]
    fn test_neg_float() {
        assert_eq!(tc("-1.0"), Ok(Type::Float));
    }

    #[test]
    fn test_neg_bool_err() {
        assert!(tc("-true").is_err());
    }

    #[test]
    fn test_not_bool() {
        assert_eq!(tc("!false"), Ok(Type::Bool));
    }

    #[test]
    fn test_not_int_err() {
        assert!(tc("!0").is_err());
    }

    #[test]
    fn test_add_int() {
        assert_eq!(tc("1 + 2"), Ok(Type::Int));
    }

    #[test]
    fn test_add_float() {
        assert_eq!(tc("1.0 + 2.0"), Ok(Type::Float));
    }

    #[test]
    fn test_add_int_float_err() {
        assert!(tc("1 + 2.0").is_err());
    }

    #[test]
    fn test_lt_int() {
        assert_eq!(tc("1 < 2"), Ok(Type::Bool));
    }

    #[test]
    fn test_eq_any_type() {
        assert_eq!(tc("true == false"), Ok(Type::Bool));
    }

    #[test]
    fn test_eq_type_mismatch_err() {
        assert!(tc("1 == true").is_err());
    }

    #[test]
    fn test_and_bool() {
        assert_eq!(tc("true && false"), Ok(Type::Bool));
    }

    #[test]
    fn test_if_ok() {
        assert_eq!(tc("if true then 1 else 0"), Ok(Type::Int));
    }

    #[test]
    fn test_if_cond_not_bool_err() {
        assert!(tc("if 1 then 2 else 3").is_err());
    }

    #[test]
    fn test_if_branch_mismatch_err() {
        assert!(tc("if true then 1 else false").is_err());
    }

    #[test]
    fn test_let_no_ann() {
        assert_eq!(tc("let x = 1 in x"), Ok(Type::Int));
    }

    #[test]
    fn test_let_with_correct_ann() {
        assert_eq!(tc("let x: Int = 1 in x"), Ok(Type::Int));
    }

    #[test]
    fn test_let_wrong_ann_err() {
        assert!(tc("let x: Bool = 1 in x").is_err());
    }

    #[test]
    fn test_lambda_identity_int() {
        assert_eq!(tc("fun (x: Int) => x"), Ok(Type::Arrow(Box::new(Type::Int), Box::new(Type::Int))));
    }

    #[test]
    fn test_lambda_multi_param() {
        assert_eq!(
            tc("fun (x: Int) (y: Int) => x + y"),
            Ok(Type::Arrow(Box::new(Type::Int), Box::new(Type::Arrow(Box::new(Type::Int), Box::new(Type::Int)))))
        );
    }

    #[test]
    fn test_let_lambda() {
        assert_eq!(
            tc("let add (x: Int) (y: Int) = x + y in add"),
            Ok(Type::Arrow(Box::new(Type::Int), Box::new(Type::Arrow(Box::new(Type::Int), Box::new(Type::Int)))))
        );
    }
    #[test]
    fn test_app_lambda() {
        assert_eq!(tc("(fun (x: Int) : Int => x) 42"), Ok(Type::Int));
    }

    #[test]
    fn test_app_wrong_arg_type_err() {
        assert!(tc("(fun (x: Int) => x) true").is_err());
    }

    #[test]
    fn test_app_not_function_err() {
        assert!(tc("42 1").is_err());
    }
    #[test]
    fn test_function_ret_mismatch() {
        assert!(tc("fun (x : Int) : Int => x <= 37").is_err())
    }

    #[test]
    fn test_letrec_factorial() {
        assert_eq!(
            tc("let rec fact (n: Int) : Int = \
                    if n == 0 then 1 else n * fact(n - 1) \
                in fact(5)"),
            Ok(Type::Int)
        );
    }

    #[test]
    fn test_letrec_wrong_body_type_err() {
        assert!(tc("let rec f (n: Int) : Int = true in f(0)").is_err());
    }

    #[test]
    fn test_ann_ok() {
        assert_eq!(tc("(1 : Int)"), Ok(Type::Int));
    }

    #[test]
    fn test_ann_mismatch_err() {
        assert!(tc("(1 : Bool)").is_err());
    }

    #[test]
    fn test_unbound_variable_err() {
        assert!(tc("foo").is_err());
    }
}
