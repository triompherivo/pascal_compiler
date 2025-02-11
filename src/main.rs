// src/main.rs

use clap::Parser;
use std::fs;
use std::collections::HashMap;
use ast::Program;
use crate::interpreter::process_declarations;

mod ast;
mod lexer;
mod parser;
mod codegen;
mod vm;
mod interpreter;

use lexer::Lexer;
use parser::Parser as PasParser;
use codegen::CodeGenerator;
use vm::VM;
use interpreter::Interpreter;
use codegen::Instruction;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the Pascal source file.
    file: String,
    /// Mode: "interpret" or "compile"
    #[arg(short, long, default_value = "interpret")]
    mode: String,
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();
    let source = fs::read_to_string(&args.file).expect("Failed to read file");

    // Lex the input.
    let mut lexer = Lexer::new(&source);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        tokens.push(token.clone());
        /* if token == lexer::Token::EOF || token == lexer::Token::Dot {
            break;
        } */
        if token == lexer::Token::EOF {
            break;
        }
    }
    if args.verbose {
        println!("Tokens:");
        for token in &tokens {
            println!("{:?}", token);
        }
    }

    

    // Create a parser and parse the program.
    let mut parser = PasParser::new(tokens);
    let program = parser.parse_program();

    if args.verbose {
        println!("AST:\n{:?}", program);
    }

    if args.mode.to_lowercase() == "compile" {
        let mut codegen = CodeGenerator::new();
    
        // (Optionally process declarations here)
        // For example, you could iterate over program.declarations
        // and generate initialization instructions, or simply ignore them.
        codegen.process_declarations(&program.declarations);
        // Generate code for the main statement block.
        codegen.gen_statement(&program.main);
        codegen.instructions.push(Instruction::Ret);


        if args.verbose {
            println!("Generated instructions:");
            for (i, instr) in codegen.instructions.iter().enumerate() {
                println!("{}: {:?}", i, instr);
            }
        }
        let mut vm = VM::new(codegen.instructions);
        vm.run();
        println!("Final globals: {:?}", vm.globals);
    } else if args.mode.to_lowercase() == "interpret" {
        let mut interp = Interpreter::new();
        process_declarations(&program, &mut interp.globals);
        let result = interp.execute_statement(&program.main);
        println!("Program result: {:?}", result);

        if args.verbose {
            println!("Final globals: {:?}", interp.globals);
        }
    }
}
