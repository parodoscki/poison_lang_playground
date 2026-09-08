#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused)]

use std::{cell::RefCell, rc::Rc};

use crate::{
    poison_abstract_syntax_tree_nodes::*, poison_lexer::Lexer, poison_parser::Parser, poison_semantic_analyzer::poison_symbol_table::*, poison_tokens::*
};

#[derive(Debug, PartialEq, Eq)]
pub enum SemaError {
    UnknownType { name: IdentifierId, span: Span },
    TypeMismatch { expected: Type, found: Type, span: Span },
    DuplicateDeclaration { name: IdentifierId, span: Span },
    CannotInferType { span: Span },
    UndefinedVariable { name: IdentifierId, span: Span },
    DuplicateParameters{ name: IdentifierId, span: Span },
    DuplicateVariable {  name: IdentifierId, span: Span }
}

pub struct SemanticAnalyzer {
    pub symbol_table: SymbolTable,
    pub expected_return_type: Option<Type>
}
// the start of my insanity

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            expected_return_type: None,
        }
    }
    pub fn analyze_statement(
        &mut self,
        ast: &Ast, 
        stmt_id: StmtId, 
        in_function: bool
    ) -> Result<(), SemaError> {
        let stmt = &ast.statements[stmt_id.0 as usize];
        match stmt {
            Statement::While { condition, body, span } => {
                let cond_ty = self.analyze_expression(ast, *condition)?;
                if cond_ty != Type::Bool {
                    return Err(SemaError::TypeMismatch { 
                        expected: Type::Bool, 
                        found: cond_ty, 
                        span: unsafe { ast.expressions.get_unchecked(condition.0 as usize).span() } 
                    });
                }
                for &inner_stmt_id in body {
                    self.analyze_statement(ast, inner_stmt_id, in_function)?;
                }
                Ok(())
            },
            Statement::Function { body, return_type, span, .. } => {
                let expected_return_type = if return_type.is_some() {
                    ast.resolved_var_decl_types[stmt_id.0 as usize].unwrap_or(Type::Void)
                } else {
                    Type::Void
                };
                
                let old_return_context = self.expected_return_type;
                self.expected_return_type = Some(expected_return_type);

                let func_name_id = ast.resolved_function_names[stmt_id.0 as usize];
                let func_symbol = Symbol {
                    name: func_name_id,
                    symbol_type: expected_return_type,
                    is_mutable: false,
                };
                let _ = self.symbol_table.insert(func_symbol);

                self.symbol_table.enter_scope();
                
                let params = &ast.resolved_function_params[stmt_id.0 as usize];
                for &(param_id, final_param_type) in params {
                    let param_symbol = Symbol {
                        name: param_id,
                        symbol_type: final_param_type,
                        is_mutable: false,
                    };

                    self.symbol_table.insert(param_symbol).map_err(|_| SemaError::DuplicateParameters {
                        name: param_id,
                        span: *span,
                    })?;
                }

                for &inner_stmt_id in body {
                    self.analyze_statement(ast, inner_stmt_id, true)?;
                }

                self.symbol_table.exit_scope();
                self.expected_return_type = old_return_context;
                Ok(())
            },
            Statement::Assignment { target, value, span, .. } => {
                let target_id = ast.resolved_assignment_targets[stmt_id.0 as usize];
                
                let target_symbol = self.symbol_table.lookup(target_id)
                    .map_err(|_| SemaError::UndefinedVariable { name: target_id, span: *span })?;
                let expected_ty = target_symbol.symbol_type;

                let value_ty = self.analyze_expression(ast, *value)?;

                if expected_ty != value_ty {
                    return Err(SemaError::TypeMismatch { expected: expected_ty, found: value_ty, span: *span });
                }
                Ok(())
            },
            Statement::Return { value, span } => {
                if !in_function {
                    return Err(SemaError::CannotInferType { span: *span }); 
                }

                let actual_return_type = match value {
                    Some(expr_id) => self.analyze_expression(ast, *expr_id)?,
                    None => Type::Void,
                };

                if let Some(expected_ty) = self.expected_return_type {
                    if actual_return_type != expected_ty { 
                        return Err(SemaError::TypeMismatch {
                            expected: expected_ty,
                            found: actual_return_type,
                            span: *span,
                        });
                    }
                }
                Ok(())
            },
            
            Statement::Block { statements, span } => {
                self.symbol_table.enter_scope();
                for &inner_stmt_id in statements {
                    self.analyze_statement(ast, inner_stmt_id, in_function)?;
                }
                self.symbol_table.exit_scope();
                Ok(())
            },
            Statement::VarDecl { is_init, is_mutable, name, initializer, span, .. } => {
                let expected_ty = unsafe { 
                    (*ast.resolved_var_decl_types.get_unchecked(stmt_id.0 as usize))
                        .ok_or(SemaError::CannotInferType { span: *span })?
                };

                if *is_init {
                    let init_ty = self.analyze_expression(ast, *initializer)?;
                    if expected_ty != init_ty {
                        return Err(SemaError::TypeMismatch { expected: expected_ty, found: init_ty, span: *span });
                    }
                }

                let name_id = unsafe { *ast.resolved_assignment_targets.get_unchecked(stmt_id.0 as usize) };

                let symbol = Symbol {
                    name: name_id,
                    symbol_type: expected_ty,
                    is_mutable: *is_mutable,
                };

                self.symbol_table.insert(symbol).map_err(|_| SemaError::DuplicateVariable { 
                    name: name_id, 
                    span: *name 
                })?;

                Ok(())
            }
            _ => Ok(())
        }
    }
    pub fn analyze_expression(
        &mut self,
        ast: &Ast,
        expr_id: ExprId,
    ) -> Result<Type, SemaError> {
        let expr = &ast.expressions[expr_id.0 as usize];        
        match expr {
            Expression::Literal { kind, span: _ } => match kind {
                LiteralKind::Integer(_) => Ok(Type::I32),
                LiteralKind::Float(_) => Ok(Type::F32),
                LiteralKind::Boolean(_) => Ok(Type::Bool),
                LiteralKind::String  => Ok(Type::Str),
            },
            Expression::Identifier { span } => {
                let name_id = unsafe { *ast.resolved_expr_identifiers.get_unchecked(expr_id.0 as usize) };
                
                match self.symbol_table.lookup(name_id) {
                    Ok(symbol) => Ok(symbol.symbol_type),
                    Err(_) => Err(SemaError::UndefinedVariable { 
                        name: name_id, 
                        span: *span 
                    }),
                }
            },
            Expression::Unary { operator, right, span } => {
                let right_ty = self.analyze_expression(ast, *right)?;

                match operator {
                    OpKind::Sub => {
                        if matches!(right_ty, Type::I32 | Type::I64 | Type::F32 | Type::F64) {
                            Ok(right_ty)
                        } else {
                            Err(SemaError::TypeMismatch {
                                expected: Type::I32,
                                found: right_ty,
                                span: *span,
                            })
                        }
                    }
                    _ => {
                        if right_ty == Type::Bool {
                            Ok(Type::Bool)
                        } else {
                            Err(SemaError::TypeMismatch {
                                expected: Type::Bool,
                                found: right_ty,
                                span: *span,
                            })
                        }
                    }
                }
            },
            Expression::Binary { left, operator, right, span } => {
                let left_ty = self.analyze_expression(ast, *left)?;
                let right_ty = self.analyze_expression(ast, *right)?;

                if left_ty != right_ty {
                    return Err(SemaError::TypeMismatch {
                        expected: left_ty,
                        found: right_ty,
                        span: *span,
                    });
                }

                match operator {
                    OpKind::Add | OpKind::Sub | OpKind::Mul | OpKind::Div => Ok(left_ty),
                    OpKind::Equal | OpKind::NotEqual | OpKind::Less | OpKind::Greater 

                    | OpKind::LessEqual | OpKind::GreaterEqual | OpKind::LogicalAnd | OpKind::LogicalOr => {
                        Ok(Type::Bool)
                    }
                    _ => Err(SemaError::TypeMismatch { 
                        expected: left_ty, 
                        found: right_ty, 
                        span: *span 
                    })
                }
            },
            Expression::Call { callee, arguments, span } => {
                let callee_ty = self.analyze_expression(ast, *callee)?;
            
                for &arg_id in arguments {
                    let _arg_ty = self.analyze_expression(ast, arg_id)?;
                }

                Ok(callee_ty)
            }
        }
    }
}