use crate::env::{Env, Value};
use crate::syntax::*;

use lalrpop_util::lalrpop_mod;
use std::{fs::read_to_string, path::PathBuf};
lalrpop_mod!(pub parser);

#[derive(Debug, Clone)]
pub struct EvalError(pub String);

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EvalError: {}", self.0)
    }
}

macro_rules! err {
    ($($t:tt)*) => { Err(EvalError(format!($($t)*))) };
}

pub type EvalResult = Result<Value, EvalError>;

pub fn eval(env: &Env, expr: &Expr) -> EvalResult {
    match expr {
        Expr::Unit => Ok(Value::Unit),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Float(f) => Ok(Value::Float(*f)),

        Expr::Var(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| EvalError(format!("unbound variable: {}", name))),

        Expr::Ann(e, _) => eval(env, e),

        Expr::UnaryOp(op, e) => {
            let v = eval(env, e)?;
            eval_unary(op, v)
        },

        Expr::BinOp(op, e1, e2) => {
            let v1 = eval(env, e1)?;
            let v2 = eval(env, e2)?;
            eval_binop(op, v1, v2)
        },

        Expr::If(cond, thn, els) => match eval(env, cond)? {
            Value::Bool(true) => eval(env, thn),
            Value::Bool(false) => eval(env, els),
            other => err!("condition must be Bool, got {}", other),
        },

        Expr::Let(name, _ty, e1, e2) => {
            let v = eval(env, e1)?;
            let env2 = env.extend(name, v);
            eval(&env2, e2)
        },

        Expr::LetRec(fname, fparams, _, fbody, body) => {
            let params: Vec<Ident> = fparams.iter().map(|(id, _)| id.clone()).collect();
            let rec_val = Value::RecClosure {
                env: env.clone(),
                fname: fname.clone(),
                params: params.clone(),
                body: (**fbody).clone(),
            };
            let env2 = env.extend(&fname, rec_val);
            eval(&env2, body)
        },

        Expr::Lambda(params, _, body) => {
            let param_names: Vec<Ident> = params.iter().map(|(id, _)| id.clone()).collect();
            Ok(Value::Closure(env.clone(), param_names, (**body).clone()))
        },

        Expr::App(func, args) => {
            let fval = eval(env, func)?;
            let mut argvs = vec![];
            for arg in args {
                let argv = eval(env, arg)?;
                argvs.push(argv);
            }
            apply(fval, argvs)
        },
    }
}

fn apply(func: Value, argvs: Vec<Value>) -> EvalResult {
    match func {
        Value::Closure(mut env, params, body) => {
            if argvs.len() > params.len() {
                return err!("too many arguments: expected {}, got {}", params.len(), argvs.len());
            }

            for (name, arg) in params.iter().zip(argvs.iter()) {
                env.insert(name.clone(), arg.clone());
            }

            if argvs.len() < params.len() {
                Ok(Value::Closure(env, params[argvs.len()..].to_vec(), body))
            } else {
                eval(&env, &body)
            }
        },

        Value::RecClosure { mut env, fname, params, body } => {
            if argvs.len() > params.len() {
                return err!("too many arguments: expected {}, got {}", params.len(), argvs.len());
            }
            env.insert(
                fname.clone(),
                Value::RecClosure {
                    env: env.clone(),
                    fname: fname.clone(),
                    params: params.clone(),
                    body: body.clone(),
                },
            );

            for (name, arg) in params.iter().zip(argvs.iter()) {
                env.insert(name.clone(), arg.clone());
            }

            if argvs.len() < params.len() {
                Ok(Value::RecClosure { env, fname, params: params[argvs.len()..].to_vec(), body })
            } else {
                eval(&env, &body)
            }
        },
        other => err!("tried to apply a non-function: {}", other),
    }
}

fn eval_unary(op: &UnaryOp, v: Value) -> EvalResult {
    match (op, v) {
        (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
        (UnaryOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
        (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
        (op, v) => err!("type error in unary {:?}: got {}", op, v),
    }
}

fn eval_binop(op: &BinOp, v1: Value, v2: Value) -> EvalResult {
    match (op, v1, v2) {
        (BinOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
        (BinOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
        (BinOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
        (BinOp::Div, Value::Int(a), Value::Int(b)) => {
            if b == 0 {
                err!("division by zero")
            } else {
                Ok(Value::Int(a / b))
            }
        },

        (BinOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
        (BinOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
        (BinOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
        (BinOp::Div, Value::Float(a), Value::Float(b)) => {
            if b == 0.0 {
                err!("division by zero (float)")
            } else {
                Ok(Value::Float(a / b))
            }
        },

        (BinOp::Eq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a == b)),
        (BinOp::Neq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a != b)),
        (BinOp::Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
        (BinOp::Leq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
        (BinOp::Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
        (BinOp::Geq, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),

        (BinOp::Eq, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a == b)),
        (BinOp::Neq, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a != b)),
        (BinOp::Lt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
        (BinOp::Leq, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
        (BinOp::Gt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
        (BinOp::Geq, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),

        (BinOp::And, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
        (BinOp::Or, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
        (BinOp::Eq, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a == b)),
        (BinOp::Neq, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a != b)),

        (BinOp::Eq, Value::Unit, Value::Unit) => Ok(Value::Bool(true)),
        (BinOp::Neq, Value::Unit, Value::Unit) => Ok(Value::Bool(false)),

        (op, v1, v2) => {
            err!("type error in {:?}: got {} and {}", op, v1, v2)
        },
    }
}

pub fn eval_top(expr: &Expr) -> EvalResult {
    eval(&Env::new(), expr)
}

pub fn eval_file(file: &PathBuf) -> EvalResult {
    if let Ok(content) = read_to_string(file.clone()) {
        let expr = parser::ExprParser::new().parse(&content).unwrap();
        eval_top(&*expr)
    } else {
        err!("read file error: {}", file.to_str().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lalrpop_util::lalrpop_mod;
    lalrpop_mod!(pub parser);

    fn run(src: &str) -> Value {
        let expr = parser::ExprParser::new().parse(src).unwrap();
        eval_top(&expr).unwrap()
    }

    fn run_err(src: &str) -> String {
        let expr = parser::ExprParser::new().parse(src).unwrap();
        eval_top(&expr).unwrap_err().0
    }

    #[test]
    fn test_literals() {
        assert_eq!(run("()"), Value::Unit);
        assert_eq!(run("true"), Value::Bool(true));
        assert_eq!(run("false"), Value::Bool(false));
        assert_eq!(run("42"), Value::Int(42));
        assert_eq!(run("3.14"), Value::Float(3.14));
    }

    #[test]
    fn test_int_arith() {
        assert_eq!(run("1 + 2"), Value::Int(3));
        assert_eq!(run("10 - 3"), Value::Int(7));
        assert_eq!(run("6 * 7"), Value::Int(42));
        assert_eq!(run("10 / 3"), Value::Int(3));
        assert_eq!(run("(1 + 2) * 3"), Value::Int(9));
        assert_eq!(run("10 - 3 - 2"), Value::Int(5));
    }

    #[test]
    fn test_float_arith() {
        assert_eq!(run("1.0 + 2.0"), Value::Float(3.0));
        assert_eq!(run("6.0 / 2.0"), Value::Float(3.0));
    }

    #[test]
    fn test_div_by_zero() {
        assert!(run_err("1 / 0").contains("division by zero"));
    }

    #[test]
    fn test_unary() {
        assert_eq!(run("-5"), Value::Int(-5));
        assert_eq!(run("-3.0"), Value::Float(-3.0));
        assert_eq!(run("!true"), Value::Bool(false));
        assert_eq!(run("!(!false)"), Value::Bool(false));
    }

    #[test]
    fn test_cmp() {
        assert_eq!(run("1 < 2"), Value::Bool(true));
        assert_eq!(run("2 > 3"), Value::Bool(false));
        assert_eq!(run("3 == 3"), Value::Bool(true));
        assert_eq!(run("3 != 4"), Value::Bool(true));
        assert_eq!(run("2 <= 2"), Value::Bool(true));
        assert_eq!(run("2 >= 3"), Value::Bool(false));
    }

    #[test]
    fn test_logic() {
        assert_eq!(run("true && false"), Value::Bool(false));
        assert_eq!(run("true || false"), Value::Bool(true));
    }

    #[test]
    fn test_if() {
        assert_eq!(run("if true then 1 else 2"), Value::Int(1));
        assert_eq!(run("if false then 1 else 2"), Value::Int(2));
        assert_eq!(run("if 1 < 2 then 10 else 20"), Value::Int(10));
        assert_eq!(run("if true then if false then 1 else 2 else 3"), Value::Int(2));
    }

    #[test]
    fn test_let() {
        assert_eq!(run("let x = 1 in x"), Value::Int(1));
        assert_eq!(run("let x : Int = 5 in x + 1"), Value::Int(6));
        assert_eq!(run("let x = 1 in let y = 2 in x + y"), Value::Int(3));
    }

    #[test]
    fn test_lambda_apply() {
        assert_eq!(run("(fun (x: Int) => x) 42"), Value::Int(42));
        assert_eq!(run("(fun (x: Int) (y: Int) => x + y) 3 4"), Value::Int(7));
    }

    #[test]
    fn test_higher_order() {
        let src = "
            let apply = fun (f: Int -> Int) (x: Int) => f x in
            let double = fun (x: Int) => x * 2 in
            apply double 21
        ";
        assert_eq!(run(src), Value::Int(42));
    }

    #[test]
    fn test_higher_order_closure() {
        let src = "
            let apply = fun (f: Int -> Int) (x: Int) => f x in
            let double = fun (x: Int) => x * 2 in
            apply double 
        ";
        let apply_closure = run(src);
        assert!(matches!(apply_closure, Value::Closure(_, _, _)));
    }

    #[test]
    fn test_factorial() {
        let src = "
            let rec fact (n: Int) : Int =
              if n == 0 then 1 else n * fact (n - 1)
            in fact 10
        ";
        assert_eq!(run(src), Value::Int(3628800));
    }

    #[test]
    fn test_fib() {
        let src = "
            let rec fib (n: Int) : Int =
              if n <= 1 then n else fib (n - 1) + fib (n - 2)
            in fib 10
        ";
        assert_eq!(run(src), Value::Int(55));
    }

    #[test]
    fn test_letrec_multi_arg() {
        let src = "
            let rec add (x: Int) (y: Int) : Int = x + y
            in add 19 23
        ";
        assert_eq!(run(src), Value::Int(42));
    }

    #[test]
    fn test_ann() {
        assert_eq!(run("(42 : Int)"), Value::Int(42));
        assert_eq!(run("(true : Bool)"), Value::Bool(true));
    }

    #[test]
    fn test_unbound_var() {
        let expr = parser::ExprParser::new().parse("x").unwrap();
        assert!(eval_top(&expr).is_err());
    }
}
