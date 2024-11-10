use crate::common::{Enriched, LineCol};
use peg::error::ParseError;

use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Num(i64),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Num(x) => write!(f, "{}", x),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
pub enum Expr<D: Default + 'static = ()> {
    Ident(String),
    Literal(Literal),
    Binary(BinaryOp, Box<Enriched<Self, D>>, Box<Enriched<Self, D>>),
}

#[derive(Debug)]
pub enum Stmt<D: Default + 'static = ()> {
    Expr(Box<Enriched<Expr<D>, D>>),
    Assign(String, Box<Enriched<Expr<D>, D>>),
}

#[derive(Debug)]
pub enum Program<D: Default + 'static = ()> {
    Stmts(Box<Vec<Enriched<Stmt<D>, D>>>),
}

peg::parser!( grammar parser() for str {
    use peg::ParseLiteral;

    rule _ = [' ' | '\n']*

    rule commasep<T>(x: rule<T>) -> Vec<T> = v:(x() ** (_ "," _)) {v}
    rule bracketed<T>(x: rule<T>) -> T = "[" _ v:x() _ "]" {v}

    rule stringlit() -> String = "\"" s:$([^ '\"']*) "\"" {s.into()}
    rule ident() -> String = "Ident" _ s:stringlit() {s}

    rule number() -> i64
        = n:$(['0'..='9']+) { n.parse().unwrap() }

    rule position() -> LineCol
        = "Just" _ "(" _ l:number() _ "," _ c:number() _ ")" { LineCol { line: l, col: c } }

    pub rule program<D: Default + 'static>() -> Program<D>
        = _ "Prog" _ "(" _ position() _ ")" _ l:bracketed(<commasep(<stmt()>)>) _ {
            Program::Stmts(Box::new(l))
        }
        / _ "Prog" _ "Nothing" _ l:bracketed(<commasep(<stmt()>)>) _ {
            Program::Stmts(Box::new(l))
        }

    pub rule stmt<D: Default + 'static>() -> Enriched<Stmt<D>, D>
        = "SAss" _ "(" _ pos:position() _ ")" _ "(" _ id:ident() _ ")" _ "(" _ e:expr() _ ")" {
            (Stmt::Assign(id, Box::new(e)), pos, D::default())
        }
        / "SExp" _ "(" _ pos:position() _ ")" _ "(" _ e:expr() _ ")" {
            (Stmt::Expr(Box::new(e)), pos, D::default())
        }

    pub rule binary_op<D: Default + 'static>(op: &BinaryOp, name: &'static str) -> Enriched<Expr<D>, D>
        = ##parse_string_literal(name) _ "(" _ pos:position() _ ")" _ "(" _ lhs:expr() _ ")" _ "(" _ rhs:expr() _ ")" {
            (Expr::Binary(op.clone(), Box::new(lhs), Box::new(rhs)), pos, D::default())
        }

    pub rule expr<D: Default + 'static>() -> Enriched<Expr<D>, D>
        = "ExpLit" _ "(" _ pos:position() _ ")" _ n:number() {
            (Expr::Literal(Literal::Num(n)), pos, D::default())
        }
        / "ExpVar" _ "(" _ pos:position() _ ")" _ "(" _ id:ident() _ ")" {
            (Expr::Ident(id), pos, D::default())
        }
        / op:binary_op(&BinaryOp::Add, "ExpAdd") {op}
        / op:binary_op(&BinaryOp::Sub, "ExpSub") {op}
        / op:binary_op(&BinaryOp::Mul, "ExpMul") {op}
        / op:binary_op(&BinaryOp::Div, "ExpDiv") {op}
});

pub fn parse<D: Default + 'static>(source: &str) -> Result<Program<D>, ParseError<peg::str::LineCol>> {
    parser::program(source)
}