use std::collections::HashMap;
use std::fmt::Display;
use crate::common::Enriched;
use crate::compiler::Compiler;
use crate::parser::{BinaryOp, Expr, Literal, Program, Stmt};

const PRINT_INT_DEF: &str = "\
@dnl = internal constant [4 x i8] c\"%d\\0A\\00\"
declare i32 @printf(i8*, ...)
define void @printInt(i32 %x) {
\t%t0 = getelementptr [4 x i8], [4 x i8]* @dnl, i32 0, i32 0
\tcall i32 (i8*, ...) @printf(i8* %t0, i32 %x)
\tret void
}";

type Register = String;

enum ExprResult {
    Register(Register),
    Value(i64),
}

impl Display for ExprResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ExprResult::Register(r) => write!(f, "{}", r),
            ExprResult::Value(v) => write!(f, "{}", v),
        }
    }
}

pub struct LLVMCompiler {
    chunks: Vec<String>,
    env: HashMap<String, Register>, // variable name -> register
    register_counter: i64,
    print_int_required: bool,
}

#[allow(dead_code)]
impl LLVMCompiler {
    pub fn new() -> Self {
        LLVMCompiler {
            chunks: Vec::new(),
            env: HashMap::new(),
            register_counter: 0,
            print_int_required: false,
        }
    }

    fn reset(&mut self) {
        self.chunks.clear();
        self.env.clear();
        self.register_counter = 0;
        self.print_int_required = false;
    }

    fn next_register(&mut self) -> String {
        self.register_counter += 1;
        format!("%t{}", self.register_counter)
    }

    fn gen_program(&mut self, program: &Program) -> String {
        self.reset();

        let Program::Stmts(stms) = program;

        self.chunks.push("define i32 @main() {".into());

        stms.iter().for_each(|stmt| {
            self.gen_stmt(stmt);
        });

        if self.print_int_required {
            self.chunks.insert(0, PRINT_INT_DEF.to_string());
        }

        self.chunks.push("\tret i32 0\n}".into());
        self.chunks.join("\n")
    }

    fn gen_stmt(&mut self, stmt: &Enriched<Stmt>) {
        let ref stmt_ast = stmt.0;

        match stmt_ast {
            Stmt::Expr(expr) => {
                let result = self.gen_expr(expr);
                self.gen_instr_call_print_int(&result.to_string());
            },
            Stmt::Assign(name, expr) => {
                let rvalue = self.gen_expr(expr);
                match rvalue {
                    ExprResult::Value(value) => {
                        let reg = self.gen_instr(&BinaryOp::Add, "0", &value.to_string());
                        self.env.insert(name.to_string(), reg);
                    }
                    ExprResult::Register(reg) => {
                        self.env.insert(name.to_string(), reg);
                    }
                }
            }
        }
    }

    fn gen_expr(&mut self, expr: &Enriched<Expr>) -> ExprResult {
        let ref expr_ast = expr.0;

        match expr_ast {
            Expr::Literal(value) => {
                let Literal::Num(v) = value;
                ExprResult::Value(*v)
            },
            Expr::Ident(name) => {
                let loc = self.env.get(name);
                match loc {
                    Some(reg ) => ExprResult::Register(reg.to_string()),
                    None => ExprResult::Value(0),
                }
            }
            Expr::Binary(op, lhs, rhs) => {
                let lhs_val = self.gen_expr(lhs).to_string();
                let rhs_val = self.gen_expr(rhs).to_string();
                ExprResult::Register(self.gen_instr(op, &lhs_val, &rhs_val))
            }
        }
    }

    fn gen_instr(&mut self, op: &BinaryOp, l: &str, r: &str) -> Register {
        let reg = self.next_register();
        let instr_name = LLVMCompiler::binary_op_to_instr_name(op);
        let instr = format!("\t{reg} = {instr_name} i32 {l}, {r}");
        self.chunks.push(instr);
        reg
    }

    fn gen_instr_call_print_int(&mut self, v: &str) {
        self.print_int_required = true;
        let instr = format!("\tcall void @printInt(i32 {v})");
        self.chunks.push(instr);
    }

    fn binary_op_to_instr_name(op: &BinaryOp) -> String {
        match op {
            BinaryOp::Add => String::from("add"),
            BinaryOp::Sub => String::from("sub"),
            BinaryOp::Mul => String::from("mul"),
            BinaryOp::Div => String::from("sdiv"),
        }
    }
}

impl Compiler for LLVMCompiler {
    type ExtendedData = ();

    fn compile(&mut self, _filename: &str, program: Program<Self::ExtendedData>) -> String {
        self.gen_program(&program)
    }
}