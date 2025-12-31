pub type Ident = String;

#[derive(Debug, PartialEq)]
pub enum Type {
    Unit,
    Bool,
    Float,
    Int,
    Arrow(Box<Type>, Box<Type>),
    Var(Ident),
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    Ann(Box<Expr>, Box<Type>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Let(Ident, Option<Type>, Box<Expr>, Box<Expr>),
    Var(Ident),
    LetRec(Fundef, Box<Expr>),
    App(Box<Expr>, Vec<Expr>),
    Lambda(Vec<(Ident, Type)>, Box<Expr>),
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, PartialEq)]
pub struct Fundef {
    pub name: Ident,
    pub args: Vec<(Ident, Type)>,
    pub ret_type: Type,
    pub body: Box<Expr>,
}

#[cfg(test)]
mod tests {
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
                }
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
                }
                _ => panic!("Unexpected structure in left operand"),
            },
            _ => panic!("Expected Expr::BinOp"),
        }
    }

    #[test]
    fn test_parse_if() {
        let expr = parser::ExprParser::new()
            .parse("if true then 1 else 0")
            .unwrap();
        match *expr {
            Expr::If(cond, then_branch, else_branch) => {
                assert_eq!(*cond, Expr::Bool(true));
                assert_eq!(*then_branch, Expr::Int(1));
                assert_eq!(*else_branch, Expr::Int(0));
            }
            _ => panic!("Expected Expr::If"),
        }
    }
    
    #[test]
    fn test_parse_lambda() {
        let expr = parser::ExprParser::new()
            .parse("fun (x: Int) (y: Int) -> x + y")
            .unwrap();
        match *expr {
            Expr::Lambda(params, body) => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0], ("x".to_string(), Type::Int));
                assert_eq!(params[1], ("y".to_string(), Type::Int));
                match *body {
                    Expr::BinOp(BinOp::Add, left, right) => {
                        assert_eq!(*left, Expr::Var("x".to_string()));
                        assert_eq!(*right, Expr::Var("y".to_string())); 
                    }
                    _ => panic!("Expected body to be a BinOp"),
                }
            }
            _ => panic!("Expected Expr::Lambda"),
        }
    }
    
    // todo : rest of tests
}
