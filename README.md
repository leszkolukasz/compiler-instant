# Instant compiler

Compiler for a simple programming language: Instant. Targets both LLVM and JVM.

## Instant overview

Instant is a very simple programming language. It has the following grammar:

```
Prog. Program ::= [Stmt] ;
SAss. Stmt ::= Ident "=" Exp;
SExp. Stmt ::= Exp ;
separator Stmt ";" ;

ExpAdd.            Exp1   ::= Exp2 "+"  Exp1 ;
ExpSub.            Exp2   ::= Exp2 "-"  Exp3 ;
ExpMul.            Exp3   ::= Exp3 "*"  Exp4 ;
ExpDiv.            Exp3   ::= Exp3 "/"  Exp4 ;
ExpLit.            Exp4   ::= Integer ;
ExpVar.            Exp4   ::= Ident ;
coercions Exp 4;
```

### Features:
- It supports simple arithmetic operations (on integers) and variable assignment.
- Statements are separated by semicolons.
- Result of expressions not assigned to a variable is printed to stdout.

### Example

```
x = 10;
x = 4 * x + 5 / 2;
x
```

will print:

```
42
```

## Technical overview

Compiler comprises the following parts:
- transpiler (Haskell) - parses the grammar and transpiles it into simpler IR
- actual compiler (Rust) - parses the IR and compiles into JVM or LLVM

Compilation target can be chosen by setting appropriate flag when building this project.

## Building

The following rules are supported:

- `make` - build project
- `make clean` - clean files

## Usage

Building the project will create two binary files:

- `insc_jvm` - compiler that targets JVM
- `insc_llvm` - compiler that targets LLVM

Running `insc_jvm input.inst` will generate file `input.j` (Jasmin bytecode) and `input.class` (JVM bytecode).

Running `insc_llvm input.inst` will generate file `input.ll` (human-readable LLVM) and `input.bc` (LLVM bitcode).

## Q&A

### Why is transpiler needed?

This project was created as part of the "Compiler design" course at the University of Warsaw. The grammar provided in the course was written for BNFC (BNF Converter) which does not support Rust. While I could have written the parser in Rust, subsequent projects in this course require more complex grammars, thus resulting in a high chance of me making a bug in the parser. As a result, I decided to use BNFC and write the transpiler in Haskell.