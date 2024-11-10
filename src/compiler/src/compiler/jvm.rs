use std::cmp::{max};
use std::collections::HashMap;
use std::path::Path;
use crate::common::Enriched;
use crate::compiler::Compiler;
use crate::parser::{BinaryOp, Expr, Literal, Program, Stmt};

const INIT: &str = "\
.method public <init>()V
\taload_0
\tinvokespecial java/lang/Object/<init>()V
\treturn
.end method";

#[derive(Default)]
pub struct NodeData {
    pub stack_size: i64
}

type Slot = i64;

pub struct JVMCompiler {
    chunks: Vec<String>,
    slot: HashMap<String, Slot>, // name -> slot
}

#[allow(dead_code)]
impl JVMCompiler {
    pub fn new() -> Self {
        JVMCompiler {
            chunks: Vec::new(),
            slot: HashMap::new(),
        }
    }

    fn reset(&mut self) {
        self.chunks.clear();
        self.slot.clear();
    }

    fn compute_stack_size(program: &mut Program<NodeData>) -> i64 {
        let Program::Stmts(stms) = program;

        stms.iter_mut().for_each(|stmt| {
            let stmt_ast = &mut stmt.0;
            let expr: &mut Enriched<Expr<NodeData>, NodeData>;
            let mut min_space = 0;

            match stmt_ast {
                Stmt::Expr(e) => {
                    expr = e;
                    min_space = 2; // expr result + getstatic
                },
                Stmt::Assign(_, e) => {
                    expr = e;
                }
            }

            stmt.2.stack_size = max(min_space, JVMCompiler::compute_stack_size_for_expr(expr));
        });

        stms.iter_mut().map(|stmt| stmt.2.stack_size ).max().unwrap_or(0)
    }

    fn compute_stack_size_for_expr(expr: &mut Enriched<Expr<NodeData>, NodeData>) -> i64 {
        let expr_ast = &mut expr.0;

        let h = match expr_ast {
            Expr::Ident(_) => 1,
            Expr::Literal(_) => 1,
            Expr::Binary(_, l, r) => {
                let l_h = JVMCompiler::compute_stack_size_for_expr(l);
                let r_h = JVMCompiler::compute_stack_size_for_expr(r);

                if l_h == r_h {
                    l_h + 1
                } else {
                    max(l_h, r_h)
                }
            }
        };

        expr.2.stack_size = h;
        h
    }

    fn get_slot_or_create(&mut self, name: &str) -> Slot {
        let len = self.slot.len() as Slot;
        self.slot.entry(name.to_string()).or_insert(len + 1).clone()
    }

    fn get_slot(&self, name: &str) -> Option<Slot> {
        self.slot.get(name).cloned()
    }

    fn gen_program(&mut self, filename: &str, program: &mut Program<NodeData>) -> String {
        self.reset();
        let stack_limit = JVMCompiler::compute_stack_size(program);

        let Program::Stmts(stms) = program;

        let classname = Path::new(filename).with_extension("").to_str().unwrap().to_string();
        self.chunks.push(format!(".class public {classname}"));
        self.chunks.push(".super java/lang/Object\n".into());
        self.chunks.push(INIT.into());
        self.chunks.push("\n.method public static main([Ljava/lang/String;)V".into());

        let limits_idx = self.chunks.len();

        stms.iter().for_each(|stmt| {
            self.gen_stmt(stmt);
        });

        let locals_limit = self.slot.len() + 1; // + 1 for main arg

        self.chunks.insert(limits_idx, format!(".limit stack {stack_limit}\n.limit locals {locals_limit}"));
        self.chunks.push("\treturn\n.end method".into());

        self.chunks.join("\n")
    }

    fn gen_stmt(&mut self, stmt: &Enriched<Stmt<NodeData>, NodeData>) {
        let stmt_ast = &stmt.0;

        match stmt_ast {
            Stmt::Expr(expr) => {
                self.gen_expr(expr);
                self.chunks.push("\tgetstatic java/lang/System/out Ljava/io/PrintStream;".into());
                self.chunks.push("\tswap".into());
                self.chunks.push("\tinvokevirtual java/io/PrintStream/println(I)V".into());
            },
            Stmt::Assign(name, expr) => {
                self.gen_expr(expr);
                let slot = self.get_slot_or_create(name);
                self.gen_instr_store(slot);
            }
        }
    }

    fn gen_expr(&mut self, expr: &Enriched<Expr<NodeData>, NodeData>) {
        let ref expr_ast = expr.0;

        match expr_ast {
            Expr::Literal(value) => {
                let Literal::Num(v) = value;
                self.gen_instr_push(*v);
            },
            Expr::Ident(name) => {
                let slot = self.get_slot(name);
                match slot {
                    Some(slot) => self.gen_instr_load(slot),
                    None => self.gen_instr_push(0)
                }
            }
            Expr::Binary(op, lhs, rhs) => {
                let l_h = lhs.2.stack_size;
                let r_h = rhs.2.stack_size;

                if l_h < r_h {
                    self.gen_expr(rhs);
                    self.gen_expr(lhs);

                    // Swap if not commutative
                    if *op == BinaryOp::Sub || *op == BinaryOp::Div {
                        self.chunks.push("swap".into());
                    }
                } else {
                    self.gen_expr(lhs);
                    self.gen_expr(rhs);
                }

                let instr = JVMCompiler::binary_op_to_instr_name(op);
                self.chunks.push(format!("\t{instr}"));
            }
        }
    }

    fn gen_instr_store(&mut self, slot: Slot) {
        assert!(slot > 0);

        if slot <= 3 {
            self.chunks.push(format!("\tistore_{slot}"))
        } else {
            self.chunks.push(format!("\tistore {slot}"));
        }
    }

    fn gen_instr_load(&mut self, slot: Slot) {
        assert!(slot > 0);

        if slot <= 3 {
            self.chunks.push(format!("\tiload_{slot}"))
        } else {
            self.chunks.push(format!("\tiload {slot}"));
        }
    }

    fn gen_instr_push(&mut self, val: i64) {
        assert!(val >= 0);

        if val <= 5 {
            self.chunks.push(format!("\ticonst_{val}"));
            return;
        }

        if val >= i8::MIN as i64 && val <= i8::MAX as i64 {
            self.chunks.push(format!("\tbipush {val}"));
        } else if val >= i16::MIN as i64 && val <= i16::MAX as i64 {
            self.chunks.push(format!("\tsipush {val}"));
        } else {
            self.chunks.push(format!("\tldc {val}"));
        }
    }

    fn binary_op_to_instr_name(op: &BinaryOp) -> String {
        match op {
            BinaryOp::Add => String::from("iadd"),
            BinaryOp::Sub => String::from("isub"),
            BinaryOp::Mul => String::from("imul"),
            BinaryOp::Div => String::from("idiv"),
        }
    }
}

impl Compiler for JVMCompiler {
    type ExtendedData = NodeData;

    fn compile(&mut self, filename: &str, mut program: Program<Self::ExtendedData>) -> String {
        self.gen_program(filename, &mut program)
    }
}