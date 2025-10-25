use crate::syntax::{BinaryOp, Expr, Program, Type, UnaryOp};
use core::fmt;
use std::collections::HashMap;

pub struct Context(HashMap<String, Type>);
impl Context {
    pub fn new() -> Self {
        Context(HashMap::new())
    }
    pub fn clone(&self) -> Self {
        Context(self.0.clone())
    }

    pub fn add(&mut self, name: String, ty: Type) {
        self.0.insert(name, ty);
    }
    pub fn get(&self, name: &str) -> Option<&Type> {
        self.0.get(name)
    }
}
#[derive(Debug, PartialEq)]
pub enum TypeError {
    ParameterTypeMismatch {
        expected: Type,
        found: Type,
        def_name: String,
    },
    ExpectFunctionType {
        found: Type,
        def_name: String,
    },
    ConditionalGuardNotBool {
        found: Type,
        def_name: String,
    },
    BranchesTypeMismatch {
        then_ty: Type,
        else_ty: Type,
        def_name: String,
    },
    UnknownTypeOfVariable {
        name: String,
        def_name: String,
    },
    DefinitionTypeMismatch {
        expected: Type,
        found: Type,
        def_name: String,
    },
    ExpectBoolType {
        found: Type,
        def_name: String,
    },
    ExpectNumberType {
        found: Type,
        def_name: String,
    },
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::ParameterTypeMismatch {
                expected,
                found,
                def_name,
            } => write!(
                f,
                "In definition '{}': Parameter type mismatch: expected {}, found {}",
                def_name, expected, found
            ),
            TypeError::ExpectFunctionType { found, def_name } => write!(
                f,
                "In definition '{}': Expected a function type, but found {}",
                def_name, found
            ),
            TypeError::ConditionalGuardNotBool { found, def_name } => write!(
                f,
                "In definition '{}': Conditional guard is not of type Bool: found {}",
                def_name, found
            ),
            TypeError::BranchesTypeMismatch {
                then_ty,
                else_ty,
                def_name,
            } => write!(
                f,
                "In definition '{}': Branches of conditional have mismatched types: then is {}, else is {}",
                def_name, then_ty, else_ty
            ),
            TypeError::UnknownTypeOfVariable { name, def_name } => write!(
                f,
                "In definition '{}': Unknown type of variable '{}'",
                def_name, name
            ),
            TypeError::DefinitionTypeMismatch {
                expected,
                found,
                def_name,
            } => write!(
                f,
                "In definition '{}': Definition type mismatch: expected {}, found {}",
                def_name, expected, found
            ),
            TypeError::ExpectBoolType { found, def_name } => write!(
                f,
                "In definition '{}': Expected a Bool type, but found {}",
                def_name, found
            ),
            TypeError::ExpectNumberType { found, def_name } => write!(
                f,
                "In definition '{}': Expected a Number type, but found {}",
                def_name, found
            ),
        }
    }
}

pub fn check_expr(expr: &Expr, ctx: &Context, def_name: String) -> Result<Type, TypeError> {
    match expr {
        Expr::Var(name) => match ctx.get(&name) {
            Some(ty) => Ok(ty.clone()),
            None => Err(TypeError::UnknownTypeOfVariable {
                name: name.clone(),
                def_name: def_name.clone(),
            }),
        },
        Expr::Number(_) => Ok(Type::Number),
        Expr::Bool(_) => Ok(Type::Bool),
        Expr::Unary { op, expr } => match op {
            UnaryOp::Minus | UnaryOp::Plus=> {
                let expr_ty = check_expr(&*expr, ctx, def_name.clone())?;
                if expr_ty != Type::Number {
                    return Err(TypeError::ExpectNumberType {
                        found: expr_ty,
                        def_name: def_name.clone(),
                    });
                }
                Ok(Type::Number)
            }
           
            UnaryOp::BitwiseNot => {
                let expr_ty = check_expr(&*expr, ctx, def_name.clone())?;
                if expr_ty != Type::Bool {
                    return Err(TypeError::ExpectBoolType {
                        found: expr_ty,
                        def_name: def_name.clone(),
                    });
                }
                Ok(Type::Number)
            }
        },
        Expr::Binary { op, left, right } => {
            let left_ty = check_expr(&*left, ctx, def_name.clone())?;
            let right_ty = check_expr(&*right, ctx, def_name.clone())?;
            match op {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                    if left_ty != Type::Number || right_ty != Type::Number {
                        return Err(TypeError::ExpectNumberType {
                            found: left_ty,
                            def_name: def_name.clone(),
                        });
                    }
                    Ok(Type::Number)
                }
                BinaryOp::BitwiseAnd | BinaryOp::BitwiseOr => {
                    if left_ty != Type::Bool || right_ty != Type::Bool {
                        return Err(TypeError::ExpectBoolType {
                            found: left_ty,
                            def_name: def_name.clone(),
                        });
                    }
                    Ok(Type::Bool)
                }
            }
        }
        Expr::If {
            pred,
            conseq,
            alter,
        } => {
            let pred_ty = check_expr(&*pred, ctx, def_name.clone())?;
            if pred_ty != Type::Bool {
                return Err(TypeError::ConditionalGuardNotBool {
                    found: pred_ty.clone(),
                    def_name: def_name.clone(),
                });
            }
            let conseq_ty = check_expr(&*conseq, ctx, def_name.clone())?;
            let alter_ty = check_expr(&*alter, ctx, def_name.clone())?;
            if conseq_ty != alter_ty {
                return Err(TypeError::BranchesTypeMismatch {
                    then_ty: conseq_ty,
                    else_ty: alter_ty,
                    def_name: def_name.clone(),
                });
            }
            Ok(conseq_ty)
        }
        Expr::Application { func, arg } => {
            let func_ty = check_expr(&*func, ctx, def_name.clone())?;
            match func_ty {
                Type::Function { input, output } => {
                    let arg_ty = check_expr(&*arg, ctx, def_name.clone())?;
                    if *input != arg_ty {
                        return Err(TypeError::ParameterTypeMismatch {
                            expected: arg_ty,
                            found: *input,
                            def_name: def_name.clone(),
                        });
                    }
                    Ok(*output)
                }
                _ => Err(TypeError::ExpectFunctionType {
                    found: func_ty.clone(),
                    def_name: def_name.clone(),
                }),
            }
        }
        Expr::Lambda {
            param,
            param_ty,
            body,
        } => {
            let mut new_ctx = (*ctx).clone();
            new_ctx.add(param.to_string(), (*param_ty).clone());
            let body_ty = check_expr(&*body, &new_ctx, def_name)?;
            Ok(Type::Function {
                input: Box::new(param_ty.clone()),
                output: Box::new(body_ty),
            })
        }
    }
}

pub fn check_program(program: &Program) -> Result<Type, Vec<TypeError>> {
    let mut ctx = Context::new();
    let mut errs = vec![];
    for def in &program.defs {
        if let Some(ty) = &def.ty {
            ctx.add(def.name.clone(), ty.clone());
        }
    }
    for def in &program.defs {
        let res = check_expr(&def.expr, &ctx, def.name.clone());
        match res {
            Ok(ty) => {
                if let Some(def_ty) = &def.ty {
                    if *def_ty != ty {
                        errs.push(TypeError::DefinitionTypeMismatch {
                            expected: def_ty.clone(),
                            found: ty.clone(),
                            def_name: def.name.clone(),
                        });
                    }
                }
                ctx.add(def.name.clone(), ty);
            }
            Err(e) => {
                errs.push(e);
            }
        }
    }
    match check_expr(&program.main, &ctx, "main".to_string()) {
        Ok(main_ty) => Ok(main_ty),
        Err(e) => {
            errs.push(e);
            Err(errs.into_iter().rev().collect())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{Expr, Type};

    #[test]
    fn test_check_expr_var() {
        let mut ctx = Context::new();
        ctx.add("x".to_string(), Type::Bool);
        let expr = Expr::Var("x".to_string());
        let ty = check_expr(&expr, &ctx, "main".to_string()).unwrap();
        assert_eq!(ty, Type::Bool);
    }

    #[test]
    fn test_check_expr_if() {
        let expr = Expr::If {
            pred: Box::new(Expr::Bool(true)),
            conseq: Box::new(Expr::Bool(false)),
            alter: Box::new(Expr::Bool(true)),
        };
        let ty = check_expr(&expr, &Context::new(), "main".to_string()).unwrap();
        assert_eq!(ty, Type::Bool);
    }

    #[test]
    fn test_check_expr_if_type_mismatch() {
        let expr = Expr::If {
            pred: Box::new(Expr::Bool(true)),
            conseq: Box::new(Expr::Bool(false)),
            alter: Box::new(Expr::Var("x".to_string())),
        };
        let mut ctx = Context::new();
        ctx.add(
            "x".to_string(),
            Type::Function {
                input: Box::new(Type::Bool),
                output: Box::new(Type::Bool),
            },
        );
        let err = check_expr(&expr, &ctx, "main".to_string()).unwrap_err();
        match err {
            TypeError::BranchesTypeMismatch {
                then_ty, else_ty, ..
            } => {
                assert_eq!(then_ty, Type::Bool);
                assert_eq!(
                    else_ty,
                    Type::Function {
                        input: Box::new(Type::Bool),
                        output: Box::new(Type::Bool)
                    }
                );
            }
            _ => panic!("Expected BranchesTypeMismatch error"),
        }
    }

    #[test]
    fn test_check_expr_application() {
        let expr = Expr::Application {
            func: Box::new(Expr::Lambda {
                param: "x".to_string(),
                param_ty: Type::Bool,
                body: Box::new(Expr::Var("x".to_string())),
            }),
            arg: Box::new(Expr::Bool(true)),
        };
        let ty = check_expr(&expr, &Context::new(), "main".to_string()).unwrap();
        assert_eq!(ty, Type::Bool);
    }
}
