use crate::syntax::{BinaryOp, DeBruijnExpr, Program, Type, UnaryOp};
use crate::type_check::TypeError;
use std::collections::HashMap;

pub struct Context(Vec<Type>);
impl Context {
    pub fn new() -> Self {
        Context(vec![])
    }
    pub fn clone(&self) -> Self {
        Context(self.0.clone())
    }

    pub fn add(&mut self, ty: Type) {
        self.0.push(ty);
    }
    pub fn get(&self, n: usize) -> Option<&Type> {
        let len = self.0.len();
        self.0.get(len - 1 - n)
    }
}

fn check_expr(
    expr: &DeBruijnExpr,
    ty_ctx: &Context,
    def_types: &HashMap<String, Type>,
    def_name: String,
) -> Result<Type, TypeError> {
    match expr {
        DeBruijnExpr::Var { index, ctx } => match ty_ctx.get(*index) {
            Some(ty) => Ok(ty.clone()),
            None => Err(TypeError::UnknownTypeOfVariable {
                name: ctx.get(*index).unwrap().clone(),
                def_name: def_name.clone(),
            }),
        },
        DeBruijnExpr::UnboundVar(name) => match def_types.get(name) {
            Some(ty) => Ok(ty.clone()),
            None => Err(TypeError::UnknownTypeOfVariable {
                name: name.clone(),
                def_name: def_name.clone(),
            }),
        },
        DeBruijnExpr::Unary { op, expr } => {
            let expr_ty = check_expr(expr, ty_ctx, def_types, def_name.clone())?;
            match op {
                UnaryOp::BitwiseNot => {
                    if expr_ty != Type::Bool {
                        return Err(TypeError::ExpectBoolType {
                            found: expr_ty,
                            def_name: def_name.clone(),
                        });
                    }
                    Ok(Type::Bool)
                }
                UnaryOp::Minus | UnaryOp::Plus=> {
                    if expr_ty != Type::Number {
                        return Err(TypeError::ExpectNumberType {
                            found: expr_ty,
                            def_name: def_name.clone(),
                        });
                    }
                    Ok(Type::Number)
                }
            }
        }
        DeBruijnExpr::Binary { op, left, right } => { 
            let left_ty = check_expr(left, ty_ctx, def_types, def_name.clone())?;
            let right_ty = check_expr(right, ty_ctx, def_types, def_name.clone())?;
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
        DeBruijnExpr::Number(_) => Ok(Type::Number),
        DeBruijnExpr::Bool(_) => Ok(Type::Bool),
        DeBruijnExpr::If {
            pred,
            conseq,
            alter,
        } => {
            let pred_ty = check_expr(pred, ty_ctx, def_types, def_name.clone())?;
            if pred_ty != Type::Bool {
                return Err(TypeError::ConditionalGuardNotBool {
                    found: pred_ty,
                    def_name: def_name.clone(),
                });
            }
            let then_ty = check_expr(conseq, ty_ctx, def_types, def_name.clone())?;
            let else_ty = check_expr(alter, ty_ctx, def_types, def_name.clone())?;
            if then_ty != else_ty {
                return Err(TypeError::BranchesTypeMismatch {
                    then_ty,
                    else_ty,
                    def_name: def_name.clone(),
                });
            }
            Ok(then_ty)
        }
        DeBruijnExpr::Application { func, arg } => {
            let func_ty = check_expr(&*func, ty_ctx, def_types, def_name.clone())?;
            match func_ty {
                Type::Function { input, output } => {
                    let arg_ty = check_expr(&*arg, ty_ctx, def_types, def_name.clone())?;
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
        DeBruijnExpr::Lambda { param_ty, body, .. } => {
            let mut new_ctx = ty_ctx.clone();
            new_ctx.add((*param_ty).clone());
            let body_ty = check_expr(body, &new_ctx, def_types, def_name.clone())?;
            Ok(Type::Function {
                input: Box::new(param_ty.clone()),
                output: Box::new(body_ty),
            })
        }
    }
}

pub fn check_program(program: &Program) -> Result<Type, Vec<TypeError>> {
    let mut def_types: HashMap<String, Type> = HashMap::new();
    let mut errs: Vec<TypeError> = vec![];
    for def in &program.defs {
        if let Some(ty) = def.ty.clone() {
            def_types.insert(def.name.clone(), ty);
        }
    }
    for def in &program.defs {
        let expr = DeBruijnExpr::from_expr(def.expr.clone());
        match check_expr(&expr, &Context::new(), &def_types, def.name.clone()) {
            Ok(inferred_ty) => {
                if let Some(expected_ty) = &def.ty {
                    if inferred_ty != *expected_ty {
                        errs.push(TypeError::DefinitionTypeMismatch {
                            expected: expected_ty.clone(),
                            found: inferred_ty.clone(),
                            def_name: def.name.clone(),
                        });
                    }
                }

                def_types.insert(def.name.clone(), inferred_ty);
            }
            Err(e) => {
                errs.push(e);
            }
        }
    }
    let expr = DeBruijnExpr::from_expr(program.main.clone());
    match check_expr(&expr, &Context::new(), &def_types, "main".to_string()) {
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
    use crate::syntax::{Expr, Program, Type};

    fn parse_program(input: &str) -> Program {
        let input = input
            .lines()
            .map(|line| line.trim_start())
            .collect::<Vec<_>>()
            .join("\n");
        let mut parser = crate::parser::Parser::from_str(&input).unwrap();
        parser.parse_program().unwrap().program
    }
    #[test]
    fn test_check_program1() {
        let str = r#"
            def zero : (t -> t) -> t -> t = \f : t -> t => \x : t => x
            def one : (t -> t) -> t -> t = \f : t -> t => \x : t => f x
            def two : (t -> t) -> t -> t = \f : t -> t => \x : t => f (f x)
            def three : (t -> t) -> t -> t = \f : t -> t => \x : t => f (f (f x))
            three 
        "#;
        let program = parse_program(str);
        let res = check_program(&program);
        assert!(res.is_ok());
    }
    #[test]
    fn test_check_program2() {
        let str = r#"
            def zero : (t -> t) -> t -> t= \f : t -> t => \x : t => x
            def id = \x :  (t -> t) -> t -> t=> x
            def x : (t -> t) -> t -> t = id zero
            x
        "#;
        let program = parse_program(str);
        let res = check_program(&program);
        assert!(res.is_ok());
    }
    #[test]
    fn test_check_program3() {
        let str = r#"
           def id : t -> t = \x : t => x
           def zero : (t -> t) -> t -> t = \f : t -> t => \x : t => x
           def one : (t -> t) -> t -> t = \f : t -> t=> \x : t => f x
           def two : (t -> t) -> t -> t = \f : t -> t => \x : t => f (f x)
           def three : (t -> t) -> t -> t = \f : t -> t => \x : t => f (f (f x))
           def f : Bool -> Bool = \b : Bool => if b then False else True
           f True
        "#;
        let program = parse_program(str);
        let res = check_program(&program);
        assert!(res.is_ok());
    }
}
