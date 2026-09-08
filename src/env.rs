use crate::syntax::{Expr, Ident};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    Closure(Env, Vec<Ident>, Expr),
    RecClosure { env: Env, fname: Ident, params: Vec<Ident>, body: Expr },
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Unit => write!(f, "()"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(x) => write!(f, "{}", x),
            Value::Closure(..) => write!(f, "<closure>"),
            Value::RecClosure { .. } => write!(f, "<rec-closure>"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Env(HashMap<Ident, Value>);

impl Env {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn extend(&self, name: &str, val: Value) -> Self {
        let mut e = self.0.clone();
        e.insert(name.to_string(), val);
        Self(e)
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.0.get(name)
    }

    pub fn insert(&mut self, name: String, val: Value) {
        self.0.insert(name, val);
    }
}
