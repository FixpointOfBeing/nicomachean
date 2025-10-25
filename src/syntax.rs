use std::fmt;
/*
 * Program := Def* Expr
 * Def := 'def' Identifier [':' Type] '=' Expr
 * Type := Atom | '(' Type '->' Type ')'
 * Expr := Var | Bool | 'if' Expr 'then' Expr 'else' Expr | 'λ' Identifier : Type '=>' Expr | (Expr) | Expr Expr
 */
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub defs: Vec<Def>,
    pub main: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Def {
    pub name: String,
    pub ty: Option<Type>,
    pub expr: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Var(String),
    Bool,
    Number,
    Function { input: Box<Type>, output: Box<Type> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Var(String),
    Bool(bool),
    Number(f64),
    If {
        pred: Box<Expr>,
        conseq: Box<Expr>,
        alter: Box<Expr>,
    },
    Application {
        func: Box<Expr>,
        arg: Box<Expr>,
    },
    Lambda {
        param: String,
        param_ty: Type,
        body: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Minus,
    Plus,
    BitwiseNot,
}
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    BitwiseAnd,
    BitwiseOr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoordProgram {
    pub program: Program,
    pub row_start: usize,
    pub col_start: usize,
    pub row_end: usize,
    pub col_end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoordDef {
    pub def: Def,
    pub row_start: usize,
    pub col_start: usize,
    pub row_end: usize,
    pub col_end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoordType {
    pub ty: Type,
    pub row_start: usize,
    pub col_start: usize,
    pub row_end: usize,
    pub col_end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CoordExpr {
    pub expr: Expr,
    pub row_start: usize,
    pub col_start: usize,
    pub row_end: usize,
    pub col_end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeBruijnExpr {
    Var {
        index: usize,
        ctx: Vec<String>,
    },
    UnboundVar(String),
    Bool(bool),
    Number(f64),
    If {
        pred: Box<DeBruijnExpr>,
        conseq: Box<DeBruijnExpr>,
        alter: Box<DeBruijnExpr>,
    },
    Application {
        func: Box<DeBruijnExpr>,
        arg: Box<DeBruijnExpr>,
    },
    Lambda {
        param: String,
        param_ty: Type,
        body: Box<DeBruijnExpr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<DeBruijnExpr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<DeBruijnExpr>,
        right: Box<DeBruijnExpr>,
    },
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for def in &self.defs {
            writeln!(f, "{def}")?;
        }
        writeln!(f, "{}", self.main)
    }
}

impl fmt::Display for Def {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ty) = &self.ty {
            writeln!(f, "def {} : {} = {}", self.name, ty, self.expr);
        }
        writeln!(f, "def {} = {}", self.name, self.expr)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Bool => write!(f, "Bool"),
            Type::Number => write!(f, "Number"),
            Type::Var(s) => write!(f, "{s}"),
            Type::Function { input, output } => write!(f, "({input} -> {output})"),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Var(s) => write!(f, "{s}"),
            Expr::Bool(true) => write!(f, "True"),
            Expr::Bool(false) => write!(f, "False"),
            Expr::Number(num) => write!(f, "{num}"),
            Expr::Unary { op, expr } => write!(f, "{op}{expr}"),
            Expr::Binary { op, left, right } => write!(f, "{left} {op} {right}"),
            Expr::If {
                pred,
                conseq,
                alter,
            } => write!(f, "if {pred} then {conseq} else {alter}"),
            Expr::Application { func, arg } => write!(f, "({func} {arg})"),
            Expr::Lambda {
                param,
                param_ty,
                body,
            } => write!(f, "(λ{param} : {param_ty} => {body})"),
        }
    }
}
impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::BitwiseAnd => "&&",
            BinaryOp::BitwiseOr => "||",
        };
        write!(f, "{s}")
    }
}
impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UnaryOp::Minus => "-",
            UnaryOp::Plus => "+",
            UnaryOp::BitwiseNot => "!",
        };
        write!(f, "{s}")
    }
}

impl fmt::Display for CoordProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}:{}-{}:{}]",
            self.program, self.row_start, self.col_start, self.row_end, self.col_end
        )
    }
}

impl fmt::Display for CoordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}:{}-{}:{}]",
            self.ty, self.row_start, self.col_start, self.row_end, self.col_end
        )
    }
}

impl fmt::Display for CoordExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}:{}-{}:{}]",
            self.expr, self.row_start, self.col_start, self.row_end, self.col_end
        )
    }
}

impl fmt::Display for DeBruijnExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeBruijnExpr::Var { index, .. } => write!(f, "{index}"),
            DeBruijnExpr::Number(num) => write!(f, "{num}"),
            DeBruijnExpr::UnboundVar(name) => write!(f, "{}", name),
            DeBruijnExpr::Bool(b) => {
                if *b {
                    write!(f, "True")
                } else {
                    write!(f, "False")
                }
            }
            DeBruijnExpr::If {
                pred,
                conseq,
                alter,
                ..
            } => {
                write!(f, "if {} then {} else {}", pred, conseq, alter)
            }
            DeBruijnExpr::Binary { op, left, right } => {
                write!(f, "{} {} {}", left, op, right)
            }
            DeBruijnExpr::Unary { op, expr } => write!(f, "{}{}", op, expr),
            DeBruijnExpr::Application { func, arg } => write!(f, "({} {})", func, arg),
            DeBruijnExpr::Lambda { body, .. } => write!(f, "(λ => {})", body),
        }
    }
}

impl Expr {
    pub fn is_value(&self) -> bool {
        match self {
            Expr::Bool(_) => true,
            Expr::Lambda { .. } => true,
            Expr::Number(_) => true,
            _ => false,
        }
    }
    pub fn is_lambda(&self) -> bool {
        matches!(self, Expr::Lambda { .. })
    }

    pub fn is_true(&self) -> bool {
        matches!(self, Expr::Bool(true))
    }

    pub fn is_false(&self) -> bool {
        matches!(self, Expr::Bool(false))
    }
}
impl DeBruijnExpr {
    pub fn is_value(&self) -> bool {
        match self {
            DeBruijnExpr::Lambda { .. } => true,
            DeBruijnExpr::Bool { .. } => true,
            DeBruijnExpr::Number(_) => true,
            _ => false,
        }
    }
    pub fn is_lambda(&self) -> bool {
        matches!(self, DeBruijnExpr::Lambda { .. })
    }
    pub fn is_true(&self) -> bool {
        match self {
            DeBruijnExpr::Bool(b) => *b,
            _ => false,
        }
    }
    pub fn is_false(&self) -> bool {
        match self {
            DeBruijnExpr::Bool(b) => !*b,
            _ => false,
        }
    }
    pub fn from_expr(expr: Expr) -> DeBruijnExpr {
        Self::from_expr_help(expr, vec![])
    }
    pub fn to_expr(&self) -> Option<Expr> {
        match self {
            DeBruijnExpr::Var { index, ctx } => {
                let len = ctx.len();
                let index1 = len - *index - 1;
                match ctx.get(index1) {
                    Some(name) => Some(Expr::Var(name.clone())),
                    None => None,
                }
            }
            DeBruijnExpr::UnboundVar(name) => Some(Expr::Var(name.clone())),
            DeBruijnExpr::Bool(b) => Some(Expr::Bool(*b)),
            DeBruijnExpr::Number(num) => Some(Expr::Number(*num)),
            DeBruijnExpr::Unary { op, expr } => {
                let expr1 = expr.to_expr()?;
                Some(Expr::Unary {
                    op: (*op).clone(),
                    expr: Box::new(expr1),
                })
            }
            DeBruijnExpr::Binary { op, left, right } => {
                let left1 = left.to_expr()?;
                let right1 = right.to_expr()?;
                Some(Expr::Binary {
                    op: (*op).clone(),
                    left: Box::new(left1),
                    right: Box::new(right1),
                })
            }

            DeBruijnExpr::If {
                pred,
                conseq,
                alter,
            } => {
                let pred1 = pred.to_expr()?;
                let conseq1 = conseq.to_expr()?;
                let alter1 = alter.to_expr()?;
                Some(Expr::If {
                    pred: Box::new(pred1),
                    conseq: Box::new(conseq1),
                    alter: Box::new(alter1),
                })
            }
            DeBruijnExpr::Application { func, arg } => {
                let func1 = func.to_expr()?;
                let arg1 = arg.to_expr()?;
                Some(Expr::Application {
                    func: Box::new(func1),
                    arg: Box::new(arg1),
                })
            }
            DeBruijnExpr::Lambda {
                param,
                param_ty,
                body,
            } => {
                let body1 = body.to_expr()?;
                Some(Expr::Lambda {
                    param: (*param).clone(),
                    param_ty: param_ty.clone(),
                    body: Box::new(body1),
                })
            }
        }
    }
    fn from_expr_help(expr: Expr, mut vars: Vec<String>) -> DeBruijnExpr {
        match expr {
            Expr::Var(name) => {
                let len = vars.len();
                match vars.clone().iter().rposition(|x| x == &name) {
                    Some(i) => {
                        let index = len - 1 - i;
                        DeBruijnExpr::Var { index, ctx: vars }
                    }
                    None => DeBruijnExpr::UnboundVar(name),
                }
            }
            Expr::Number(num) => DeBruijnExpr::Number(num),
            Expr::Bool(b) => DeBruijnExpr::Bool(b),
            Expr::If {
                pred,
                conseq,
                alter,
            } => {
                let pred1 = Self::from_expr_help(*pred, vars.clone());
                let conseq1 = Self::from_expr_help(*conseq, vars.clone());
                let alter = Self::from_expr_help(*alter, vars.clone());
                DeBruijnExpr::If {
                    pred: Box::new(pred1),
                    conseq: Box::new(conseq1),
                    alter: Box::new(alter),
                }
            }
            Expr::Unary { op, expr } => {
                let expr1 = Self::from_expr_help(*expr, vars.clone());
                DeBruijnExpr::Unary {
                    op,
                    expr: Box::new(expr1),
                }
            }
            Expr::Binary { op, left, right } => {
                let left1 = Self::from_expr_help(*left, vars.clone());
                let right1 = Self::from_expr_help(*right, vars.clone());
                DeBruijnExpr::Binary {
                    op,
                    left: Box::new(left1),
                    right: Box::new(right1),
                }
            }
            Expr::Application { func, arg } => {
                let func1 = Self::from_expr_help(*func, vars.clone());
                let arg1 = Self::from_expr_help(*arg, vars.clone());
                DeBruijnExpr::Application {
                    func: Box::new(func1),
                    arg: Box::new(arg1),
                }
            }
            Expr::Lambda {
                param,
                param_ty,
                body,
            } => {
                vars.push(param.clone());
                let body1 = Self::from_expr_help(*body, vars.clone());
                DeBruijnExpr::Lambda {
                    param,
                    param_ty,
                    body: Box::new(body1),
                }
            }
        }
    }
}
#[cfg(test)]
mod context_debruijn_tests {
    use super::*;

    fn test_from_expr(expr: Expr, expected: DeBruijnExpr) {
        let result = DeBruijnExpr::from_expr(expr.clone());
        assert_eq!(result, expected);
        // Also test to_expr round-trip
        let expr_back = result.to_expr().unwrap();
        assert_eq!(expr_back, expr);
    }

    #[test]
    fn test_simple_identity_lambda() {
        // λs : Bool => λz : Bool => z
        let expr = Expr::Lambda {
            param: "s".to_string(),
            param_ty: Type::Bool,
            body: Box::new(Expr::Lambda {
                param: "z".to_string(),
                param_ty: Type::Bool,
                body: Box::new(Expr::Var("z".to_string())),
            }),
        };
        let expected = DeBruijnExpr::Lambda {
            param: "s".to_string(),
            param_ty: Type::Bool,
            body: Box::new(DeBruijnExpr::Lambda {
                param: "z".to_string(),
                param_ty: Type::Bool,
                body: Box::new(DeBruijnExpr::Var {
                    index: 0,
                    ctx: vec!["s".to_string(), "z".to_string()],
                }),
            }),
        };
        test_from_expr(expr, expected);
    }

    #[test]
    fn test_simple_application_lambda() {
        // λs : Bool => λz : Bool => s (s z)
        let expr = Expr::Lambda {
            param: "s".to_string(),
            param_ty: Type::Bool,
            body: Box::new(Expr::Lambda {
                param: "z".to_string(),
                param_ty: Type::Bool,
                body: Box::new(Expr::Application {
                    func: Box::new(Expr::Var("s".to_string())),
                    arg: Box::new(Expr::Application {
                        func: Box::new(Expr::Var("s".to_string())),
                        arg: Box::new(Expr::Var("z".to_string())),
                    }),
                }),
            }),
        };
        let expected = DeBruijnExpr::Lambda {
            param: "s".to_string(),
            param_ty: Type::Bool,
            body: Box::new(DeBruijnExpr::Lambda {
                param: "z".to_string(),
                param_ty: Type::Bool,
                body: Box::new(DeBruijnExpr::Application {
                    func: Box::new(DeBruijnExpr::Var {
                        index: 1,
                        ctx: vec!["s".to_string(), "z".to_string()],
                    }),
                    arg: Box::new(DeBruijnExpr::Application {
                        func: Box::new(DeBruijnExpr::Var {
                            index: 1,
                            ctx: vec!["s".to_string(), "z".to_string()],
                        }),
                        arg: Box::new(DeBruijnExpr::Var {
                            index: 0,
                            ctx: vec!["s".to_string(), "z".to_string()],
                        }),
                    }),
                }),
            }),
        };
        test_from_expr(expr, expected);
    }

    #[test]
    fn test_church_add_lambda() {
        // λm : _ => λn : _ => λs : _ => λz : _ => m s (n z s)
        let expr = Expr::Lambda {
            param: "m".to_string(),
            param_ty: Type::Var("_".to_string()),
            body: Box::new(Expr::Lambda {
                param: "n".to_string(),
                param_ty: Type::Var("_".to_string()),
                body: Box::new(Expr::Lambda {
                    param: "s".to_string(),
                    param_ty: Type::Var("_".to_string()),
                    body: Box::new(Expr::Lambda {
                        param: "z".to_string(),

                        param_ty: Type::Var("_".to_string()),
                        body: Box::new(Expr::Application {
                            func: Box::new(Expr::Application {
                                func: Box::new(Expr::Var("m".to_string())),
                                arg: Box::new(Expr::Var("s".to_string())),
                            }),
                            arg: Box::new(Expr::Application {
                                func: Box::new(Expr::Application {
                                    func: Box::new(Expr::Var("n".to_string())),
                                    arg: Box::new(Expr::Var("z".to_string())),
                                }),
                                arg: Box::new(Expr::Var("s".to_string())),
                            }),
                        }),
                    }),
                }),
            }),
        };
        let expected = DeBruijnExpr::Lambda {
            param: "m".to_string(),
            param_ty: Type::Var("_".to_string()),
            body: Box::new(DeBruijnExpr::Lambda {
                param: "n".to_string(),
                param_ty: Type::Var("_".to_string()),
                body: Box::new(DeBruijnExpr::Lambda {
                    param: "s".to_string(),
                    param_ty: Type::Var("_".to_string()),
                    body: Box::new(DeBruijnExpr::Lambda {
                        param: "z".to_string(),
                        param_ty: Type::Var("_".to_string()),
                        body: Box::new(DeBruijnExpr::Application {
                            func: Box::new(DeBruijnExpr::Application {
                                func: Box::new(DeBruijnExpr::Var {
                                    index: 3,
                                    ctx: vec![
                                        "m".to_string(),
                                        "n".to_string(),
                                        "s".to_string(),
                                        "z".to_string(),
                                    ],
                                }),
                                arg: Box::new(DeBruijnExpr::Var {
                                    index: 1,
                                    ctx: vec![
                                        "m".to_string(),
                                        "n".to_string(),
                                        "s".to_string(),
                                        "z".to_string(),
                                    ],
                                }),
                            }),
                            arg: Box::new(DeBruijnExpr::Application {
                                func: Box::new(DeBruijnExpr::Application {
                                    func: Box::new(DeBruijnExpr::Var {
                                        index: 2,
                                        ctx: vec![
                                            "m".to_string(),
                                            "n".to_string(),
                                            "s".to_string(),
                                            "z".to_string(),
                                        ],
                                    }),
                                    arg: Box::new(DeBruijnExpr::Var {
                                        index: 0,
                                        ctx: vec![
                                            "m".to_string(),
                                            "n".to_string(),
                                            "s".to_string(),
                                            "z".to_string(),
                                        ],
                                    }),
                                }),
                                arg: Box::new(DeBruijnExpr::Var {
                                    index: 1,
                                    ctx: vec![
                                        "m".to_string(),
                                        "n".to_string(),
                                        "s".to_string(),
                                        "z".to_string(),
                                    ],
                                }),
                            }),
                        }),
                    }),
                }),
            }),
        };
        test_from_expr(expr, expected);
    }

    #[test]
    fn test_y_combinator_lambda() {
        // λf : _  => (λx : _ => f (λy : _ => (x x) y)) (λx : _ => f (λy : _ => (x x) y))
        let expr = Expr::Lambda {
            param: "f".to_string(),
            param_ty: Type::Var("_".to_string()),
            body: Box::new(Expr::Application {
                func: Box::new(Expr::Lambda {
                    param: "x".to_string(),
                    param_ty: Type::Var("_".to_string()),
                    body: Box::new(Expr::Application {
                        func: Box::new(Expr::Var("f".to_string())),
                        arg: Box::new(Expr::Lambda {
                            param_ty: Type::Var("_".to_string()),
                            param: "y".to_string(),
                            body: Box::new(Expr::Application {
                                func: Box::new(Expr::Application {
                                    func: Box::new(Expr::Var("x".to_string())),
                                    arg: Box::new(Expr::Var("x".to_string())),
                                }),
                                arg: Box::new(Expr::Var("y".to_string())),
                            }),
                        }),
                    }),
                }),
                arg: Box::new(Expr::Lambda {
                    param: "x".to_string(),
                    param_ty: Type::Var("_".to_string()),
                    body: Box::new(Expr::Application {
                        func: Box::new(Expr::Var("f".to_string())),
                        arg: Box::new(Expr::Lambda {
                            param: "y".to_string(),
                            param_ty: Type::Var("_".to_string()),
                            body: Box::new(Expr::Application {
                                func: Box::new(Expr::Application {
                                    func: Box::new(Expr::Var("x".to_string())),
                                    arg: Box::new(Expr::Var("x".to_string())),
                                }),
                                arg: Box::new(Expr::Var("y".to_string())),
                            }),
                        }),
                    }),
                }),
            }),
        };

        let result = DeBruijnExpr::from_expr(expr.clone());
        let expr_back = result.to_expr().unwrap();
        assert_eq!(expr_back, expr);
    }

    #[test]
    fn test_shadowing_lambda() {
        // (λx : _=> (λx : _ => x)) (λx : _ => x)
        let expr = Expr::Application {
            func: Box::new(Expr::Lambda {
                param: "x".to_string(),
                param_ty: Type::Var("_".to_string()),
                body: Box::new(Expr::Lambda {
                    param: "x".to_string(),
                    param_ty: Type::Var("_".to_string()),
                    body: Box::new(Expr::Var("x".to_string())),
                }),
            }),
            arg: Box::new(Expr::Lambda {
                param: "x".to_string(),
                param_ty: Type::Var("_".to_string()),
                body: Box::new(Expr::Var("x".to_string())),
            }),
        };

        let result = DeBruijnExpr::from_expr(expr.clone());
        let expr_back = result.to_expr().unwrap();
        assert_eq!(expr_back, expr);
    }
}
