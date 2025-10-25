use crate::syntax::{BinaryOp, Expr, Program, UnaryOp};
use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    str,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Debug, PartialEq)]
pub enum EvalError {
    UnboundVariable(String),
}
impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::UnboundVariable(var) => write!(f, "Eval error: unbound variable: {}", var),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Env(HashMap<String, Expr>);
impl Env {
    pub fn new() -> Self {
        Env(HashMap::new())
    }
    pub fn get(&self, key: &str) -> Option<&Expr> {
        self.0.get(key)
    }
    pub fn insert(&mut self, key: String, value: Expr) {
        self.0.insert(key, value);
    }
}
static GENSYM_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn gensym(prefix: &str) -> String {
    let id = GENSYM_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}{}", prefix, id)
}

fn frees(expr: &Expr) -> HashSet<String> {
    match expr {
        Expr::Var(x) => HashSet::from([x.clone()]),
        Expr::Bool(_) => HashSet::new(),
        Expr::Number(_) => HashSet::new(),
        Expr::Unary { expr, .. } => frees(expr),
        Expr::Binary { left, right, .. } => {
            let mut frees1 = frees(left);
            let frees2 = frees(right);
            frees1.extend(frees2);
            frees1
        }
        Expr::Application { func, arg } => {
            let mut frees1 = frees(func);
            let frees2 = frees(arg);
            frees1.extend(frees2);
            frees1
        }
        Expr::Lambda { param, body, .. } => {
            let mut frees1 = frees(body);
            frees1.remove(param);
            frees1
        }
        Expr::If {
            pred,
            conseq,
            alter,
        } => {
            let mut frees1 = frees(pred);
            let frees2 = frees(conseq);
            let frees3 = frees(alter);
            frees1.extend(frees2);
            frees1.extend(frees3);
            frees1
        }
    }
}
pub fn substitute(body: &Expr, x: &str, s: &Expr) -> Expr {
    match &body {
        Expr::Var(var) => {
            if var == x {
                s.clone()
            } else {
                body.clone()
            }
        }
        Expr::Bool(_) => (*body).clone(),
        Expr::Number(_) => (*body).clone(),
        Expr::Binary { op, left, right } => {
            let left = substitute(left, x, s);
            let right = substitute(right, x, s);
            Expr::Binary {
                op: (*op).clone(),
                left: Box::new(left),
                right: Box::new(right),
            }
        }
        Expr::Unary { op, expr } => {
            let expr = substitute(expr, x, s);
            Expr::Unary {
                op: (*op).clone(),
                expr: Box::new(expr),
            }
        }
        Expr::Application { func, arg } => {
            let func1 = substitute(&**func, x, s);
            let arg1 = substitute(&**arg, x, s);
            Expr::Application {
                func: Box::new(func1),
                arg: Box::new(arg1),
            }
        }
        Expr::If {
            pred,
            conseq,
            alter,
        } => {
            let pred = Box::new(substitute(pred, x, s));
            let conseq = Box::new(substitute(conseq, x, s));
            let alter = Box::new(substitute(alter, x, s));
            Expr::If {
                pred,
                conseq,
                alter,
            }
        }
        Expr::Lambda {
            param,
            param_ty,
            body: inner_body,
        } => {
            if param == x {
                // x is shadowed, do not substitute inside
                Expr::Lambda {
                    param: param.clone(),
                    param_ty: param_ty.clone(),
                    body: inner_body.clone(),
                }
            } else if frees(s).contains(param) {
                // param conflicts, do alpha-renaming
                let new_param = gensym(param);
                let renamed_body = substitute(&*inner_body, param, &Expr::Var(new_param.clone()));
                let new_lambda = Expr::Lambda {
                    param: new_param.clone(),
                    param_ty: param_ty.clone(),
                    body: Box::new(renamed_body),
                };
                substitute(&new_lambda, x, s)
            } else {
                Expr::Lambda {
                    param: param.clone(),
                    param_ty: param_ty.clone(),
                    body: Box::new(substitute(&*inner_body, x, s)),
                }
            }
        }
    }
}

pub fn step_expr(expr: Expr, env: &Env) -> Result<Option<Expr>, EvalError> {
    let expr = match expr {
        Expr::Application { func, arg } => {
            let func = match *func {
                Expr::Var(name) => match env.get(&name) {
                    Some(e) => e.clone(),
                    None => Expr::Var(name),
                },
                _ => *func,
            };
            let arg = match *arg {
                Expr::Var(name) => match env.get(&name) {
                    Some(e) => e.clone(),
                    None => Expr::Var(name),
                },
                _ => *arg,
            };
            Expr::Application {
                func: Box::new(func),
                arg: Box::new(arg),
            }
        }
        _ => expr,
    };
    match expr {
        Expr::Application { func, arg } if (*func).is_lambda() && (*arg).is_value() => {
            if let Expr::Lambda { param, body, .. } = *func {
                Ok(Some(substitute(&*body, &param, &*arg)))
            } else {
                unreachable!()
            }
        }
        Expr::Application { func, arg } if (*func).is_value() => {
            let next = step_expr(*arg, env)?;
            Ok(next.map(|next| Expr::Application {
                func,
                arg: Box::new(next),
            }))
        }

        Expr::Application { func, arg } if (*arg).is_value() => {
            let next = step_expr(*func, env)?;
            Ok(next.map(|next| Expr::Application {
                func: Box::new(next),
                arg,
            }))
        }
        Expr::If { pred, conseq, .. } if pred.is_true() => Ok(Some(*conseq)),
        Expr::If { pred, alter, .. } if pred.is_false() => Ok(Some(*alter)),
        Expr::If {
            pred,
            conseq,
            alter,
        } => {
            let next = step_expr(*pred, env)?;
            Ok(next.map(|next| Expr::If {
                pred: Box::new(next),
                conseq,
                alter,
            }))
        }
        Expr::Var(name) => match env.get(&name) {
            Some(e) => Ok(Some(e.clone())),
            None => Err(EvalError::UnboundVariable(name)),
        },
        Expr::Unary { op, expr } => match *expr {
            Expr::Number(n) => match op {
                UnaryOp::Minus => match *expr {
                    Expr::Number(n) => Ok(Some(Expr::Number(-n))),
                    _ => unreachable!(),
                },
                UnaryOp::Plus => match *expr {
                    Expr::Number(n) => Ok(Some(Expr::Number(n))),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            },
            Expr::Bool(b) => match op {
                UnaryOp::BitwiseNot => Ok(Some(Expr::Bool(!b))),
                _ => unreachable!(),
            },
            _ => {
                let next = step_expr(*expr, env)?;
                Ok(next.map(|next| Expr::Unary {
                    op,
                    expr: Box::new(next),
                }))
            }
        },

        Expr::Binary { op, left, right } => match *left {
            Expr::Number(l) => match *right {
                Expr::Number(r) => match op {
                    BinaryOp::Add => Ok(Some(Expr::Number(l + r))),
                    BinaryOp::Sub => Ok(Some(Expr::Number(l - r))),
                    BinaryOp::Mul => Ok(Some(Expr::Number(l * r))),
                    BinaryOp::Div => Ok(Some(Expr::Number(l / r))),
                    _ => unreachable!(),
                },
                _ => {
                    let next = step_expr(*right, env)?;
                    Ok(next.map(|next| Expr::Binary {
                        op,
                        left: Box::new(Expr::Number(l)),
                        right: Box::new(next),
                    }))
                }
            },
            Expr::Bool(l) => match *right {
                Expr::Bool(r) => match op {
                    BinaryOp::BitwiseAnd=> Ok(Some(Expr::Bool(l && r))),
                    BinaryOp::BitwiseOr => Ok(Some(Expr::Bool(l || r))),
                    _ => unreachable!(),
                },
                _ => {
                    let next = step_expr(*right, env)?;
                    Ok(next.map(|next| Expr::Binary {
                        op,
                        left: Box::new(Expr::Bool(l)),
                        right: Box::new(next),
                    }))
                }
            },
            _ => {
                let next = step_expr(*left, env)?;
                Ok(next.map(|next| Expr::Binary {
                    op,
                    left: Box::new(next),
                    right,
                }))
            }
        },
        _ => Ok(None),
    }
}

pub fn eval_expr(mut expr: Expr, env: &Env) -> Result<Expr, EvalError> {
    while let Some(next) = step_expr(expr.clone(), env)? {
        expr = next;
    }
    Ok(expr)
}
pub fn eval_program(prog: Program) -> Result<Expr, EvalError> {
    let mut env = Env::new();
    for def in prog.defs {
        let expr = def.expr;
        let name = def.name;
        let value = eval_expr(expr, &env)?;
        env.insert(name, value);
    }
    eval_expr(prog.main, &env)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::*;
    #[test]
    fn test_eval_simple_bool() {
        let expr = Expr::Bool(true);
        assert_eq!(eval_expr(expr.clone(), &Env::new()).unwrap(), expr);
    }

    #[test]
    fn test_eval_var() {
        let expr = Expr::Var("x".to_string());
        let res = eval_expr(expr.clone(), &Env::new());
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err(),
            EvalError::UnboundVariable("x".to_string())
        );
    }

    #[test]
    fn test_eval_lambda_is_value() {
        let expr = Expr::Lambda {
            param: "x".to_string(),
            param_ty: Type::Var("_".to_string()),
            body: Box::new(Expr::Var("x".to_string())),
        };
        assert_eq!(eval_expr(expr.clone(), &Env::new()).unwrap(), expr);
    }

    #[test]
    fn test_eval_application_simple() {
        // (λx : Bool => x) true ==> true
        let expr = Expr::Application {
            func: Box::new(Expr::Lambda {
                param: "x".to_string(),
                param_ty: Type::Bool,
                body: Box::new(Expr::Var("x".to_string())),
            }),
            arg: Box::new(Expr::Bool(true)),
        };
        assert_eq!(eval_expr(expr, &Env::new()).unwrap(), Expr::Bool(true));
    }

    #[test]
    fn test_eval_nested_application() {
        // ((λx : Bool => λy : Bool => x) true) false ==> true
        let expr = Expr::Application {
            func: Box::new(Expr::Application {
                func: Box::new(Expr::Lambda {
                    param: "x".to_string(),
                    param_ty: Type::Bool,
                    body: Box::new(Expr::Lambda {
                        param: "y".to_string(),
                        param_ty: Type::Bool,
                        body: Box::new(Expr::Var("x".to_string())),
                    }),
                }),
                arg: Box::new(Expr::Bool(true)),
            }),
            arg: Box::new(Expr::Bool(false)),
        };
        let expected = Expr::Bool(true);
        assert_eq!(eval_expr(expr, &Env::new()).unwrap(), expected);
    }

    #[test]
    fn test_eval_application_with_non_value_arg() {
        // (λx : Bool => x) ((λy : Bool => y) true) ==> true
        let expr = Expr::Application {
            func: Box::new(Expr::Lambda {
                param: "x".to_string(),
                param_ty: Type::Bool,
                body: Box::new(Expr::Var("x".to_string())),
            }),
            arg: Box::new(Expr::Application {
                func: Box::new(Expr::Lambda {
                    param: "y".to_string(),
                    param_ty: Type::Bool,
                    body: Box::new(Expr::Var("y".to_string())),
                }),
                arg: Box::new(Expr::Bool(true)),
            }),
        };
        assert_eq!(eval_expr(expr, &Env::new()).unwrap(), Expr::Bool(true));
    }

    #[test]
    fn test_eval_lambda_shadowing() {
        // (λx : Bool => λx : Bool => x) true ==> λx => x
        let expr = Expr::Application {
            func: Box::new(Expr::Lambda {
                param: "x".to_string(),
                param_ty: Type::Bool,
                body: Box::new(Expr::Lambda {
                    param: "x".to_string(),
                    param_ty: Type::Bool,
                    body: Box::new(Expr::Var("x".to_string())),
                }),
            }),
            arg: Box::new(Expr::Bool(true)),
        };
        let expected = Expr::Lambda {
            param: "x".to_string(),
            param_ty: Type::Bool,
            body: Box::new(Expr::Var("x".to_string())),
        };
        assert_eq!(eval_expr(expr, &Env::new()).unwrap(), expected);
    }
    #[test]
    fn test_eval_if_true() {
        // if True then False else True ==> False
        let expr = Expr::If {
            pred: Box::new(Expr::Bool(true)),
            conseq: Box::new(Expr::Bool(false)),
            alter: Box::new(Expr::Bool(true)),
        };
        assert_eq!(eval_expr(expr, &Env::new()).unwrap(), Expr::Bool(false));
    }

    #[test]
    fn test_eval_if_false() {
        // if False then True else False ==> False
        let expr = Expr::If {
            pred: Box::new(Expr::Bool(false)),
            conseq: Box::new(Expr::Bool(true)),
            alter: Box::new(Expr::Bool(false)),
        };
        assert_eq!(eval_expr(expr, &Env::new()).unwrap(), Expr::Bool(false));
    }

    #[test]
    fn test_eval_if_nested() {
        // if (if True then False else True) then True else False ==> False
        let expr = Expr::If {
            pred: Box::new(Expr::If {
                pred: Box::new(Expr::Bool(true)),
                conseq: Box::new(Expr::Bool(false)),
                alter: Box::new(Expr::Bool(true)),
            }),
            conseq: Box::new(Expr::Bool(true)),
            alter: Box::new(Expr::Bool(false)),
        };
        assert_eq!(eval_expr(expr, &Env::new()).unwrap(), Expr::Bool(false));
    }

    #[test]
    fn test_eval_if_with_application_predicate() {
        // if ((λx : Bool => x) True) then False else True ==> False
        let expr = Expr::If {
            pred: Box::new(Expr::Application {
                func: Box::new(Expr::Lambda {
                    param: "x".to_string(),
                    param_ty: Type::Bool,
                    body: Box::new(Expr::Var("x".to_string())),
                }),
                arg: Box::new(Expr::Bool(true)),
            }),
            conseq: Box::new(Expr::Bool(false)),
            alter: Box::new(Expr::Bool(true)),
        };
        assert_eq!(eval_expr(expr, &Env::new()).unwrap(), Expr::Bool(false));
    }
}
