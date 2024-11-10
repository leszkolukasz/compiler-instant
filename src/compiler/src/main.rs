mod common;
mod error;
mod compiler;
mod parser;

#[allow(unused_imports)]
use crate::compiler::jvm::JVMCompiler;
#[allow(unused_imports)]
use crate::compiler::llvm::LLVMCompiler;
use crate::compiler::Compiler;
use crate::parser::Program;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::{env, process};
use crate::common::LineCol;
use crate::error::{report_error, Error};

#[cfg(feature = "llvm")]
type NodeData = ();
#[cfg(feature = "jvm")]
type NodeData = compiler::jvm::NodeData;

fn run_program(path: &str, args: Vec<String>) -> (String, String) {
    let result = Command::new(path).args(args).output().expect("Failed to execute process");
    let stdout = String::from_utf8(result.stdout).expect("Failed to read stdout");
    let stderr = String::from_utf8(result.stderr).expect("Failed to read stderr");


    if !result.status.success() {
        eprintln!("{stdout}");
        eprintln!("{stderr}");
        eprintln!("Process {path} exited with non-zero exit code");
        process::exit(1);
    };

    (stdout, stderr)
}

fn run_transpiler(file_path: &str) -> String {
    run_program("./lib/transpiler", vec![file_path.to_string()]).0
}

fn run_parser(source: &str) -> Program<NodeData> {
    let parser_result = parser::parse::<NodeData>(source);

    if let Err(err) = parser_result {
        report_error("Unknown".into(), source, Error {
            msg: "Parsing haskell output failed".into(),
            pos: LineCol {
                line: err.location.line as i64,
                col: err.location.column as i64,
            }
        });
        process::exit(1);
    }

    parser_result.unwrap()
}


fn export_code(file_path: &Path, code: &str) -> String {
    let extension = if cfg!(feature = "llvm") {
        "ll"
    } else {
        "j"
    };

    let path = file_path.with_extension(extension);

    let mut file = File::create(&path).unwrap();
    file.write_all(code.as_bytes()).unwrap();

    path.to_str().unwrap().to_string()
}

fn run_compiler(file_path: &str, program: Program<NodeData>) -> String {
    #[cfg(feature = "jvm")]
    let mut compiler = JVMCompiler::new();

    #[cfg(feature = "llvm")]
    let mut compiler = LLVMCompiler::new();

    compiler.compile(file_path, program)
}

fn run_postprocessing(file_path: &str) {
    let path = Path::new(file_path);
    if cfg!(feature = "llvm") {
        let args: Vec<String> = vec![
            "-o".into(),
            path.with_extension("bc").to_str().unwrap().into(),
            path.to_str().unwrap().into(),
        ];

        run_program("llvm-as", args);
    } else {
        let mut parent = path.parent().unwrap().to_str().unwrap().to_string();
        if parent.len() == 0 { parent = ".".into(); };

        let args: Vec<String> = vec![
            "-jar".into(),
            "lib/jasmin.jar".into(),
            "-d".into(),
            parent,
            path.to_str().unwrap().into()
        ];

        // Jasmin seems to always exit with code 0
        let (_, stderr) = run_program("java", args);
        if stderr.len() != 0 {
            eprintln!("{stderr}");
            eprintln!("Jasmin failed quietly");
            process::exit(1);
        }
    }
}

fn main() {
    let file_path_str = env::args().nth(1).expect("Expected file argument");
    let file_path = Path::new(&file_path_str);
    let file_name = file_path.file_name().unwrap().to_str().unwrap();

    let source = run_transpiler(&file_path_str);
    let program = run_parser(&source);
    let code = run_compiler(file_name, program);

    let exported_file = export_code(&file_path, &code);
    run_postprocessing(&exported_file);
}