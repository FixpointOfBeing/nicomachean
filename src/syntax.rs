pub type Ident = String;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Unit,
    Bool,
    Float,
    Int,
    Arrow(Box<Type>, Box<Type>),
    Var(Ident),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    Var(Ident),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    Ann(Box<Expr>, Type),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Let(Ident, Option<Type>, Box<Expr>, Box<Expr>),
    LetRec(
        Ident,              // function name
        Vec<(Ident, Type)>, // function arguments with their types
        Type,               // function return type
        Box<Expr>,          // function body
        Box<Expr>,          // expression after the let rec
    ),
    App(Box<Expr>, Vec<Expr>),
    Lambda(Vec<(Ident, Type)>, Option<Type>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Eq,
    Neq,
    Lt,
    Gt,
    Leq,
    Geq,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

// ----------------------------------------------------------------------------------------------------
impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Unit => {
                write!(f, "Unit")
            },
            Type::Bool => {
                write!(f, "Bool")
            },
            Type::Int => {
                write!(f, "Int")
            },
            Type::Float => {
                write!(f, "Float")
            },
            Type::Var(name) => {
                write!(f, "{}", name)
            },
            Type::Arrow(from, to) => {
                let from_str = match **from {
                    Type::Arrow(_, _) => format!("({})", from),
                    _ => format!("{}", from),
                };
                write!(f, "{} -> {}", from_str, to)
            },
        }
    }
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op_str = match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::And => "&&",
            BinOp::Or => "||",
            BinOp::Eq => "=",
            BinOp::Neq => "<>",
            BinOp::Lt => "<",
            BinOp::Gt => ">",
            BinOp::Leq => "<=",
            BinOp::Geq => ">=",
        };
        write!(f, "{}", op_str)
    }
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op_str = match self {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
        };
        write!(f, "{}", op_str)
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Unit => {
                write!(f, "()")
            },
            Expr::Bool(b) => {
                write!(f, "{}", b)
            },
            Expr::Int(n) => {
                write!(f, "{}", n)
            },
            Expr::Float(fl) => {
                write!(f, "{}", fl)
            },
            Expr::Var(name) => {
                write!(f, "{}", name)
            },
            Expr::BinOp(op, left, right) => {
                write!(f, "({} {} {})", left, op, right)
            },
            Expr::UnaryOp(op, expr) => {
                write!(f, "({}{})", op, expr)
            },
            Expr::Ann(expr, ty) => {
                write!(f, "({} : {})", expr, ty)
            },
            Expr::If(cond, then_branch, else_branch) => {
                write!(f, "if {} then {} else {}", cond, then_branch, else_branch)
            },
            Expr::Let(name, ann, val, body) => {
                if let Some(ty) = ann {
                    write!(f, "let {}: {} = {} in {}", name, ty, val, body)
                } else {
                    write!(f, "let {} = {} in {}", name, val, body)
                }
            },
            Expr::LetRec(fname, fparams, fret_ty, fbody, body) => {
                let params_str = fparams
                    .iter()
                    .map(|(param_name, param_ty)| format!("({}: {})", param_name, param_ty))
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "let rec {} {} : {} = {} in {}", fname, params_str, fret_ty, fbody, body)
            },
            Expr::App(func, args) => {
                let args_str = args
                    .iter()
                    .map(|arg| format!("{}", arg))
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "({} {})", func, args_str)
            },
            Expr::Lambda(params, ty, body) => {
                let params_str = params
                    .iter()
                    .map(|(param_name, param_ty)| format!("({}: {})", param_name, param_ty))
                    .collect::<Vec<_>>()
                    .join(" ");
                if let Some(ret_ty) = ty {
                    write!(f, "fun {} : {} => {}", params_str, ret_ty, body)
                } else {
                    write!(f, "fun {} => {}", params_str, body)
                }
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::syntax::Type::{Int, Unit};

    use super::*;
    use lalrpop_util::lalrpop_mod;
    lalrpop_mod!(pub parser);

    #[test]
    fn test_parse_int() {
        let expr = parser::ExprParser::new().parse("42").unwrap();
        match *expr {
            Expr::Int(n) => assert_eq!(n, 42),
            _ => panic!("Expected Expr::Int"),
        }
    }

    #[test]
    fn test_parse_binop1() {
        let expr = parser::ExprParser::new().parse("1 + 2 * 4").unwrap();
        match *expr {
            Expr::BinOp(BinOp::Add, left, right) => match (*left, *right) {
                (Expr::Int(1), Expr::BinOp(BinOp::Mul, left, right)) => {
                    assert_eq!(*left, Expr::Int(2));
                    assert_eq!(*right, Expr::Int(4));
                },
                _ => panic!("Unexpected structure in right operand"),
            },
            _ => panic!("Expected Expr::BinOp"),
        }
    }

    #[test]
    fn test_parse_binop2() {
        let expr = parser::ExprParser::new().parse("(3 - 5) / 2").unwrap();
        match *expr {
            Expr::BinOp(BinOp::Div, left, right) => match (*left, *right) {
                (Expr::BinOp(BinOp::Sub, left_sub, right_sub), Expr::Int(2)) => {
                    assert_eq!(*left_sub, Expr::Int(3));
                    assert_eq!(*right_sub, Expr::Int(5));
                },
                _ => {
                    panic!("Unexpected structure in left operand")
                },
            },
            _ => panic!("Expected Expr::BinOp"),
        }
    }

    #[test]
    fn test_parse_if() {
        let expr = parser::ExprParser::new().parse("if true then 1 else 0").unwrap();
        match *expr {
            Expr::If(cond, then_branch, else_branch) => {
                assert_eq!(*cond, Expr::Bool(true));
                assert_eq!(*then_branch, Expr::Int(1));
                assert_eq!(*else_branch, Expr::Int(0));
            },
            _ => panic!("Expected Expr::If"),
        }
    }

    #[test]
    fn test_parse_lambda() {
        let expr = parser::ExprParser::new()
            .parse("fun (x: Int) (y: Int) : Int => x + y")
            .unwrap();
        match *expr {
            Expr::Lambda(params, ty, body) => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0], ("x".to_string(), Type::Int));
                assert_eq!(params[1], ("y".to_string(), Type::Int));
                assert_eq!(ty, Some(Type::Int));
                match *body {
                    Expr::BinOp(BinOp::Add, left, right) => {
                        assert_eq!(*left, Expr::Var("x".to_string()));
                        assert_eq!(*right, Expr::Var("y".to_string()));
                    },
                    _ => panic!("Expected body to be a BinOp"),
                }
            },
            _ => panic!("Expected Expr::Lambda"),
        }
    }

    #[test]
    fn test_parse_unit() {
        let expr = parser::ExprParser::new().parse("()").unwrap();
        assert_eq!(*expr, Expr::Unit);
    }

    #[test]
    fn test_parse_bool_true() {
        let expr = parser::ExprParser::new().parse("true").unwrap();
        assert_eq!(*expr, Expr::Bool(true));
    }

    #[test]
    fn test_parse_bool_false() {
        let expr = parser::ExprParser::new().parse("false").unwrap();
        assert_eq!(*expr, Expr::Bool(false));
    }

    #[test]
    fn test_parse_float() {
        let expr = parser::ExprParser::new().parse("3.14").unwrap();
        match *expr {
            Expr::Float(f) => assert!((f - 3.14).abs() < 1e-10),
            _ => panic!("Expected Expr::Float"),
        }
    }

    #[test]
    fn test_parse_negative_int() {
        let expr = parser::ExprParser::new().parse("-42").unwrap();
        assert_eq!(*expr, Expr::UnaryOp(UnaryOp::Neg, Box::new(Expr::Int(42))));
    }

    #[test]
    fn test_parse_negative_float() {
        let expr = parser::ExprParser::new().parse("-1.5").unwrap();
        match *expr {
            Expr::UnaryOp(UnaryOp::Neg, inner) => match *inner {
                Expr::Float(f) => assert!((f - 1.5).abs() < 1e-10),
                _ => panic!("Expected Expr::Float inside Neg"),
            },
            _ => panic!("Expected Expr::UnaryOp(Neg, ...)"),
        }
    }

    #[test]
    fn test_parse_var() {
        let expr = parser::ExprParser::new().parse("foo").unwrap();
        assert_eq!(*expr, Expr::Var("foo".to_string()));
    }

    #[test]
    fn test_parse_unary_not() {
        let expr = parser::ExprParser::new().parse("!true").unwrap();
        assert_eq!(*expr, Expr::UnaryOp(UnaryOp::Not, Box::new(Expr::Bool(true))));
    }

    #[test]
    fn test_parse_unary_neg() {
        let expr = parser::ExprParser::new().parse("-x").unwrap();
        assert_eq!(*expr, Expr::UnaryOp(UnaryOp::Neg, Box::new(Expr::Var("x".to_string()))));
    }

    #[test]
    fn test_parse_binop_sub() {
        let expr = parser::ExprParser::new().parse("10 - 3").unwrap();
        assert_eq!(*expr, Expr::BinOp(BinOp::Sub, Box::new(Expr::Int(10)), Box::new(Expr::Int(3))));
    }

    #[test]
    fn test_parse_binop_mul() {
        let expr = parser::ExprParser::new().parse("6 * 7").unwrap();
        assert_eq!(*expr, Expr::BinOp(BinOp::Mul, Box::new(Expr::Int(6)), Box::new(Expr::Int(7))));
    }

    #[test]
    fn test_parse_binop_div() {
        let expr = parser::ExprParser::new().parse("8 / 2").unwrap();
        assert_eq!(*expr, Expr::BinOp(BinOp::Div, Box::new(Expr::Int(8)), Box::new(Expr::Int(2))));
    }

    #[test]
    fn test_parse_binop_eq() {
        let expr = parser::ExprParser::new().parse("x == y").unwrap();
        assert_eq!(
            *expr,
            Expr::BinOp(BinOp::Eq, Box::new(Expr::Var("x".to_string())), Box::new(Expr::Var("y".to_string())))
        );
    }

    #[test]
    fn test_parse_binop_neq() {
        let expr = parser::ExprParser::new().parse("x != y").unwrap();
        assert_eq!(
            *expr,
            Expr::BinOp(BinOp::Neq, Box::new(Expr::Var("x".to_string())), Box::new(Expr::Var("y".to_string())))
        );
    }

    #[test]
    fn test_parse_binop_lt() {
        let expr = parser::ExprParser::new().parse("1 < 2").unwrap();
        assert_eq!(*expr, Expr::BinOp(BinOp::Lt, Box::new(Expr::Int(1)), Box::new(Expr::Int(2))));
    }

    #[test]
    fn test_parse_binop_gt() {
        let expr = parser::ExprParser::new().parse("2 > 1").unwrap();
        assert_eq!(*expr, Expr::BinOp(BinOp::Gt, Box::new(Expr::Int(2)), Box::new(Expr::Int(1))));
    }

    #[test]
    fn test_parse_binop_leq() {
        let expr = parser::ExprParser::new().parse("1 <= 2").unwrap();
        assert_eq!(*expr, Expr::BinOp(BinOp::Leq, Box::new(Expr::Int(1)), Box::new(Expr::Int(2))));
    }

    #[test]
    fn test_parse_binop_geq() {
        let expr = parser::ExprParser::new().parse("2 >= 1").unwrap();
        assert_eq!(*expr, Expr::BinOp(BinOp::Geq, Box::new(Expr::Int(2)), Box::new(Expr::Int(1))));
    }

    #[test]
    fn test_parse_binop_and() {
        let expr = parser::ExprParser::new().parse("true && false").unwrap();
        assert_eq!(*expr, Expr::BinOp(BinOp::And, Box::new(Expr::Bool(true)), Box::new(Expr::Bool(false))));
    }

    #[test]
    fn test_parse_binop_or() {
        let expr = parser::ExprParser::new().parse("true || false").unwrap();
        assert_eq!(*expr, Expr::BinOp(BinOp::Or, Box::new(Expr::Bool(true)), Box::new(Expr::Bool(false))));
    }

    #[test]
    fn test_precedence_add_vs_mul() {
        let expr = parser::ExprParser::new().parse("2 + 3 * 4").unwrap();
        match *expr {
            Expr::BinOp(BinOp::Add, left, right) => {
                assert_eq!(*left, Expr::Int(2));
                assert_eq!(*right, Expr::BinOp(BinOp::Mul, Box::new(Expr::Int(3)), Box::new(Expr::Int(4))));
            },
            _ => panic!("Expected Add at top level"),
        }
    }

    #[test]
    fn test_precedence_compare_vs_arith() {
        let expr = parser::ExprParser::new().parse("1 + 2 < 3 + 4").unwrap();
        match *expr {
            Expr::BinOp(BinOp::Lt, left, right) => {
                assert_eq!(*left, Expr::BinOp(BinOp::Add, Box::new(Expr::Int(1)), Box::new(Expr::Int(2))));
                assert_eq!(*right, Expr::BinOp(BinOp::Add, Box::new(Expr::Int(3)), Box::new(Expr::Int(4))));
            },
            _ => panic!("Expected Lt at top level"),
        }
    }

    #[test]
    fn test_left_associativity_sub() {
        let expr = parser::ExprParser::new().parse("10 - 3 - 2").unwrap();
        match *expr {
            Expr::BinOp(BinOp::Sub, left, right) => {
                assert_eq!(*left, Expr::BinOp(BinOp::Sub, Box::new(Expr::Int(10)), Box::new(Expr::Int(3))));
                assert_eq!(*right, Expr::Int(2));
            },
            _ => panic!("Expected Sub at top level"),
        }
    }

    #[test]
    fn test_parse_ann_arrow_type() {
        let expr = parser::ExprParser::new()
            .parse("((fun (x: Int) : Int => x) : Int -> Int)")
            .unwrap();
        match *expr {
            Expr::Ann(inner, ty) => {
                assert_eq!(ty, Type::Arrow(Box::new(Type::Int), Box::new(Type::Int)));
                match *inner {
                    Expr::Lambda(params, _, body) => {
                        assert_eq!(params[0], ("x".to_string(), Type::Int));
                        assert_eq!(*body, Expr::Var("x".to_string()));
                    },
                    _ => panic!("Expected Lambda inside Ann"),
                }
            },
            _ => panic!("Expected Expr::Ann"),
        }
    }

    #[test]
    fn test_parse_let_no_annotation() {
        let expr = parser::ExprParser::new().parse("let x = 1 in x").unwrap();
        match *expr {
            Expr::Let(name, ann, val, body) => {
                assert_eq!(name, "x");
                assert_eq!(ann, None);
                assert_eq!(*val, Expr::Int(1));
                assert_eq!(*body, Expr::Var("x".to_string()));
            },
            _ => panic!("Expected Expr::Let"),
        }
    }

    #[test]
    fn test_parse_let_with_annotation() {
        let expr = parser::ExprParser::new().parse("let x: Int = 1 in x").unwrap();
        match *expr {
            Expr::Let(name, ann, val, body) => {
                assert_eq!(name, "x");
                assert_eq!(ann, Some(Type::Int));
                assert_eq!(*val, Expr::Int(1));
                assert_eq!(*body, Expr::Var("x".to_string()));
            },
            _ => panic!("Expected Expr::Let"),
        }
    }

    #[test]
    fn test_let_lambda() {
        let expr = parser::ExprParser::new()
            .parse("let f (b: Bool) (x: Int) (y: Int) : Int = if b then x + y else x - y in f true 3 5")
            .unwrap();
        match *expr {
            Expr::Let(name, ann, val, body) => {
                assert_eq!(name, "f");
                assert_eq!(
                    ann,
                    Some(Type::Arrow(
                        Box::new(Type::Bool),
                        Box::new(Type::Arrow(
                            Box::new(Type::Int),
                            Box::new(Type::Arrow(Box::new(Type::Int), Box::new(Type::Int)))
                        ))
                    ))
                );
                match *val {
                    Expr::Lambda(params, ty, lambda_body) => {
                        assert_eq!(params.len(), 3);
                        assert_eq!(params[0], ("b".to_string(), Type::Bool));
                        assert_eq!(params[1], ("x".to_string(), Type::Int));
                        assert_eq!(params[2], ("y".to_string(), Type::Int));
                        assert_eq!(ty, Some(Type::Int));
                        match *lambda_body {
                            Expr::If(cond, then_branch, else_branch) => {
                                assert_eq!(*cond, Expr::Var("b".to_string()));
                                assert_eq!(
                                    *then_branch,
                                    Expr::BinOp(
                                        BinOp::Add,
                                        Box::new(Expr::Var("x".to_string())),
                                        Box::new(Expr::Var("y".to_string()))
                                    )
                                );
                                assert_eq!(
                                    *else_branch,
                                    Expr::BinOp(
                                        BinOp::Sub,
                                        Box::new(Expr::Var("x".to_string())),
                                        Box::new(Expr::Var("y".to_string()))
                                    )
                                );
                            },
                            _ => panic!("Expected If in lambda body"),
                        }
                    },
                    _ => panic!("Expected Lambda as value in Let"),
                }
                match *body {
                    Expr::App(func, args) => {
                        assert_eq!(args.len(), 3);
                        assert_eq!(args[0], Expr::Bool(true));
                        assert_eq!(args[1], Expr::Int(3));
                        assert_eq!(args[2], Expr::Int(5));
                        match *func {
                            Expr::Var(ref name) if name == "f" => {},
                            _ => panic!("Expected Var 'f' as function in App"),
                        }
                    },
                    _ => panic!("Expected App as body of Let"),
                }
            },
            _ => panic!("Expected Expr::Let"),
        }
    }
    #[test]
    fn test_parse_let_nested() {
        let expr = parser::ExprParser::new()
            .parse("let x = 1 in let y = 2 in x + y")
            .unwrap();
        match *expr {
            Expr::Let(x, _, x_val, body) => {
                assert_eq!(x, "x");
                assert_eq!(*x_val, Expr::Int(1));
                match *body {
                    Expr::Let(y, _, y_val, inner) => {
                        assert_eq!(y, "y");
                        assert_eq!(*y_val, Expr::Int(2));
                        assert_eq!(
                            *inner,
                            Expr::BinOp(
                                BinOp::Add,
                                Box::new(Expr::Var("x".to_string())),
                                Box::new(Expr::Var("y".to_string()))
                            )
                        );
                    },
                    _ => panic!("Expected inner Let"),
                }
            },
            _ => panic!("Expected Expr::Let"),
        }
    }

    #[test]
    fn test_parse_letrec_factorial() {
        let expr = parser::ExprParser::new()
            .parse("let rec fact (n: Int) : Int = if n == 0 then 1 else n * fact(n - 1) in fact(5)")
            .unwrap();
        match *expr {
            Expr::LetRec(fname, fargs, fret_ty, _, body) => {
                assert_eq!(fname, "fact");
                assert_eq!(fargs.len(), 1);
                assert_eq!(fargs[0], ("n".to_string(), Type::Int));
                assert_eq!(fret_ty, Type::Int);
                assert_eq!(*body, Expr::App(Box::new(Expr::Var("fact".to_string())), vec![Expr::Int(5)]));
            },
            _ => panic!("Expected Expr::LetRec"),
        }
    }

    #[test]
    fn test_parse_letrec_multi_args() {
        let expr = parser::ExprParser::new()
            .parse("let rec add (x: Int) (y: Int) : Int = x + y in add 1 2")
            .unwrap();
        match *expr {
            Expr::LetRec(fname, fargs, fret_ty, _, _) => {
                assert_eq!(fname, "add");
                assert_eq!(fargs.len(), 2);
                assert_eq!(fret_ty, Type::Int);
            },
            _ => panic!("Expected Expr::LetRec"),
        }
    }

    #[test]
    fn test_parse_app_single_arg() {
        let expr = parser::ExprParser::new().parse("f 1").unwrap();
        assert_eq!(*expr, Expr::App(Box::new(Expr::Var("f".to_string())), vec![Expr::Int(1)]));
    }

    #[test]
    fn test_parse_app_multi_args() {
        let expr = parser::ExprParser::new().parse("f 1 2 3").unwrap();
        assert_eq!(
            *expr,
            Expr::App(Box::new(Expr::Var("f".to_string())), vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)])
        );
    }

    #[test]
    fn test_parse_app_lambda_immediately() {
        let expr = parser::ExprParser::new()
            .parse("(fun (x: Int) : Int => x) 42")
            .unwrap();
        match *expr {
            Expr::App(func, args) => {
                assert_eq!(args, vec![Expr::Int(42)]);
                match *func {
                    Expr::Lambda(params, ty, body) => {
                        assert_eq!(ty, Some(Int));
                        assert_eq!(params[0], ("x".to_string(), Type::Int));
                        assert_eq!(*body, Expr::Var("x".to_string()));
                    },
                    _ => panic!("Expected Lambda as function"),
                }
            },
            _ => panic!("Expected Expr::App"),
        }
    }

    #[test]
    fn test_parse_lambda_single_param() {
        let expr = parser::ExprParser::new().parse("fun (x: Bool) => !x").unwrap();
        match *expr {
            Expr::Lambda(params, ty, body) => {
                assert_eq!(ty, None);
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], ("x".to_string(), Type::Bool));
                assert_eq!(*body, Expr::UnaryOp(UnaryOp::Not, Box::new(Expr::Var("x".to_string()))));
            },
            _ => panic!("Expected Expr::Lambda"),
        }
    }

    #[test]
    fn test_parse_lambda_unit_param() {
        let expr = parser::ExprParser::new().parse("fun (x: Unit) : Unit => ()").unwrap();
        match *expr {
            Expr::Lambda(params, ty, body) => {
                assert_eq!(ty, Some(Unit));
                assert_eq!(params[0], ("x".to_string(), Type::Unit));
                assert_eq!(*body, Expr::Unit);
            },
            _ => panic!("Expected Expr::Lambda"),
        }
    }

    #[test]
    fn test_parse_if_nested() {
        let expr = parser::ExprParser::new()
            .parse("if true then if false then 1 else 2 else 3")
            .unwrap();
        match *expr {
            Expr::If(cond, then_branch, else_branch) => {
                assert_eq!(*cond, Expr::Bool(true));
                assert_eq!(*else_branch, Expr::Int(3));
                match *then_branch {
                    Expr::If(inner_cond, inner_then, inner_else) => {
                        assert_eq!(*inner_cond, Expr::Bool(false));
                        assert_eq!(*inner_then, Expr::Int(1));
                        assert_eq!(*inner_else, Expr::Int(2));
                    },
                    _ => panic!("Expected nested If"),
                }
            },
            _ => panic!("Expected Expr::If"),
        }
    }

    #[test]
    fn test_parse_if_with_binop_condition() {
        let expr = parser::ExprParser::new().parse("if x > 0 then x else -x").unwrap();
        match *expr {
            Expr::If(cond, then_branch, else_branch) => {
                assert_eq!(*cond, Expr::BinOp(BinOp::Gt, Box::new(Expr::Var("x".to_string())), Box::new(Expr::Int(0))));
                assert_eq!(*then_branch, Expr::Var("x".to_string()));
                assert_eq!(*else_branch, Expr::UnaryOp(UnaryOp::Neg, Box::new(Expr::Var("x".to_string()))));
            },
            _ => panic!("Expected Expr::If"),
        }
    }

    #[test]
    fn test_parse_type_arrow_right_assoc() {
        let expr = parser::ExprParser::new().parse("(f : Int -> Int -> Bool)").unwrap();
        match *expr {
            Expr::Ann(_, ty) => {
                assert_eq!(
                    ty,
                    Type::Arrow(Box::new(Type::Int), Box::new(Type::Arrow(Box::new(Type::Int), Box::new(Type::Bool))))
                );
            },
            _ => panic!("Expected Expr::Ann"),
        }
    }

    #[test]
    fn test_parse_type_var() {
        let expr = parser::ExprParser::new().parse("(x : a)").unwrap();
        match *expr {
            Expr::Ann(_, ty) => {
                assert_eq!(ty, Type::Var("a".to_string()));
            },
            _ => panic!("Expected Expr::Ann with type var"),
        }
    }

    #[test]
    fn test_parse_error_empty() {
        assert!(parser::ExprParser::new().parse("").is_err());
    }

    #[test]
    fn test_parse_error_unmatched_paren() {
        assert!(parser::ExprParser::new().parse("(1 + 2").is_err());
    }

    #[test]
    fn test_parse_error_missing_else() {
        assert!(parser::ExprParser::new().parse("if true then 1").is_err());
    }

    #[test]
    fn test_parse_error_dangling_op() {
        assert!(parser::ExprParser::new().parse("1 +").is_err());
    }
}
