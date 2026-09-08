use compiler_frontend::{
    poison_abstract_syntax_tree_nodes::*,
    poison_lexer::Lexer,
    poison_parser::Parser,
    poison_semantic_analyzer::poison_analyzer::SemanticAnalyzer,
    poison_semantic_analyzer::poison_symbol_table::Interner,
};
use compiler_backend::llvm::main_codegen::llvm_ast_walker::LLVMWalkerCodeGenerator;

pub fn compile_pipeline(source_code: &str) -> Result<(), String> {
    let mut interner = Interner::new();
    
    let lexer = Lexer::new(source_code, &mut interner);
    let mut parser = Parser::new(lexer);
    let mut ast = Ast::new();

    let root_statements = parser.parse_program(&mut ast);

    if !parser.errors.is_empty() {
        parser.report_errors();
        return Err("Syntax validation failed. Halting compilation pass.".to_string());
    }

    ast.prepare_caches(source_code, &parser.tokens_trace);

    let mut analyzer = SemanticAnalyzer::new();
    for &stmt_id in &root_statements {
        analyzer.analyze_statement(&ast, stmt_id, false)
            .map_err(|err| format!("Semantic Validation Error: {:?}", err))?;
    }

    let mut codegen = LLVMWalkerCodeGenerator::new();

    unsafe {
        let printf_ref = codegen.declare_printf();
        
        let printf_id = interner.intern("printf");
        
        codegen.global_functions.insert(printf_id, printf_ref);
    }
    for &stmt_id in &root_statements {
        codegen.walk_statement(&ast, stmt_id, &interner);
    }

    codegen.backend.dump();

    codegen.backend.dispose();

    Ok(())
}
pub fn compile_pipeline_and_get_string(source_code: &str) -> Result<String, String> {
    let mut interner = Interner::new();
    
    let lexer = Lexer::new(source_code, &mut interner);
    let mut parser = Parser::new(lexer);
    let mut ast = Ast::new();

    let root_statements = parser.parse_program(&mut ast);

    if !parser.errors.is_empty() {
        parser.report_errors();
        return Err("Syntax validation failed. Halting compilation pass.".to_string());
    }

    ast.prepare_caches(source_code, &parser.tokens_trace);

    let mut analyzer = SemanticAnalyzer::new();
    for &stmt_id in &root_statements {
        analyzer.analyze_statement(&ast, stmt_id, false)
            .map_err(|err| format!("Semantic Validation Error: {:?}", err))?;
    }

    let mut codegen = LLVMWalkerCodeGenerator::new();

    unsafe {
        let printf_ref = codegen.declare_printf();
        
        let printf_id = interner.intern("printf");
        
        codegen.global_functions.insert(printf_id, printf_ref);
    }
    for &stmt_id in &root_statements {
        codegen.walk_statement(&ast, stmt_id, &interner);
    }

    let string =unsafe { codegen.backend.get_llvm_ir_string() };

    codegen.backend.dispose();

    Ok(string)
}
use std::{env, fs};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let file_code = fs::read_to_string(&args[1])?;
    
    println!("{:?}", compile_pipeline(&file_code));
    Ok(())
}
