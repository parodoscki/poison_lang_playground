use crate::{poison_semantic_analyzer::poison_symbol_table::{IdentifierId, Type}, poison_tokens::{Span, Token}};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq,Hash)]
pub struct ExprId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StmtId(pub u32);

#[derive(Debug)]
pub enum Statement {
    Expression(ExprId),
    Block {
        statements: Vec<StmtId>,
        span: Span,
    },
    Structure {
        name: Span,
        fields: Vec<StructField>,
        span: Span,
    },
    Function {
        name: Span,
        parameters: Vec<Param>, // Holds parameter identifier spans
        body: Vec<StmtId>,
        return_type: Option<Span>, // FIX: Holds the optional return type span (like 'i32')
        attribute: Option<Span>,   // FIX: Holds the attribute span (like 'gpu' or 'main')
        span: Span,
    },
    VarDecl {
        is_init: bool, 
        is_mutable: bool,
        name: Span,
        explicit_type: Option<Span>,
        initializer: ExprId,
        span: Span,
    },
    While{
        condition: ExprId,
        body: Vec<StmtId>,
        span: Span,
    },
    For {
        variable: Span,
        range_start: ExprId,
        range_end: ExprId,
        is_inclusive: bool,
        step_by: ExprId,
        body: Vec<StmtId>,
        span: Span,
    },
    Assignment {
        target: Span,
        operator: OpKind,
        value: ExprId,
        span: Span,
    },
    Return {
        value: Option<ExprId>,
        span: Span,
    },
    Connect {
        target: Span,
        span: Span,
    },
    If {
        condition: ExprId, 
        then_branch: Vec<StmtId>,
        else_branch: Option<Vec<StmtId>>,
        span: Span,
    },
}
#[derive(Debug, Clone)]
pub struct Param {
    pub name: Span,
    pub param_type: Span,
}
#[derive(Debug)]
pub enum Expression {
    Literal {
        kind: LiteralKind,
        span: Span,
    },
    Identifier {
        span: Span,
    },
    Binary {
        left: ExprId,
        operator: OpKind,
        right: ExprId,
        span: Span,
    },
    Call {
        callee: ExprId,
        arguments: Vec<ExprId>,
        span: Span,
    },
    Unary {
        operator: OpKind,
        right: ExprId,
        span: Span,
    },
}
impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::Literal { span, .. } => *span,
            Expression::Identifier { span } => *span,
            Expression::Binary { span, .. } => *span,
            Expression::Call { span, .. } => *span,
            Expression::Unary { span, .. } => *span,
        }
    }
}

pub struct Identifier {
    pub span: Span,
}
#[derive(Debug)]
pub struct StructField {
    pub name: Span,
    pub field_type: Span,
    pub span: Span,
}
#[derive(Debug)]
pub enum LiteralKind {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String,
}
#[derive(Debug)]
pub enum OpKind {
    Add, Sub, Mul, Div,
    Equal, NotEqual, Less, Greater,
    LessEqual, GreaterEqual, LogicalAnd,
    AddressOf,
    LogicalOr,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    Assign,
}



pub struct Ast {
    pub statements: Vec<Statement>,
    pub expressions: Vec<Expression>,
    pub root_statements: Vec<StmtId>, 

    pub resolved_string_literals: HashMap<ExprId, IdentifierId>, 
    pub resolved_expr_identifiers: Vec<IdentifierId>,
    pub resolved_var_decl_types: Vec<Option<Type>>,
    pub resolved_assignment_targets: Vec<IdentifierId>,
    pub resolved_function_names: Vec<IdentifierId>,
    pub resolved_function_params: Vec<Vec<(IdentifierId, Type)>>,
}
impl Ast {
    pub fn print_tree(&self, source: &str) {
        println!("--- ABSTRACT SYNTAX TREE ---");
        for &stmt_id in &self.root_statements {
            self.print_statement(source, stmt_id, 0);
        }
        println!("----------------------------");
    }

    fn print_statement(&self, source: &str, stmt_id: StmtId, indent: usize) {
        let pad = "  ".repeat(indent);
        let stmt = &self.statements[stmt_id.0 as usize];

        match stmt {
            Statement::Function { name, parameters, body, return_type, .. } => {
                let func_name = &source[name.start..name.end];
                let ret_ty = return_type.map_or("void", |span| &source[span.start..span.end]);
                
                println!("{pad}Function: @{func_name} (returns {ret_ty})");
                
                // Print Parameters
                for param in parameters {
                    let p_name = &source[param.name.start..param.name.end];
                    let p_type = &source[param.param_type.start..param.param_type.end];
                    println!("{pad}  Param: {p_name} : {p_type}");
                }
                
                // Print Inner Body Block Elements
                println!("{pad}  Body:");
                for &inner_stmt_id in body {
                    self.print_statement(source, inner_stmt_id, indent + 2);
                }
            }
            Statement::VarDecl { is_init, is_mutable, name, explicit_type, initializer, .. } => {
                let var_name = &source[name.start..name.end];
                let mut_str = if *is_mutable { "mut " } else { "" };
                let ty_str = explicit_type.map_or("Inferred".to_string(), |span| source[span.start..span.end].to_string());
                
                println!("{pad}VarDecl: {mut_str}{var_name} ({ty_str})");
                if *is_init {
                    println!("{pad}  Initializer:");
                    self.print_expression(source, *initializer, indent + 2);
                }
            }
            Statement::Assignment { target, operator, value, .. } => {
                let var_name = &source[target.start..target.end];
                println!("{pad}Assignment: {var_name} ({:?})", operator);
                self.print_expression(source, *value, indent + 1);
            }
            Statement::Return { value, .. } => {
                println!("{pad}Return");
                if let Some(expr_id) = value {
                    self.print_expression(source, *expr_id, indent + 1);
                }
            }
            Statement::Block { statements, .. } => {
                println!("{pad}Block:");
                for &inner_stmt_id in statements {
                    self.print_statement(source, inner_stmt_id, indent + 1);
                }
            }
            Statement::If { condition, then_branch, else_branch, .. } => {
                println!("{pad}If Condition:");
                self.print_expression(source, *condition, indent + 1);
                println!("{pad}Then Branch:");
                for &stmt in then_branch { self.print_statement(source, stmt, indent + 1); }
                if let Some(else_stmts) = else_branch {
                    println!("{pad}Else Branch:");
                    for &stmt in else_stmts { self.print_statement(source, stmt, indent + 1); }
                }
            }
            Statement::While { condition, body, .. } => {
                println!("{pad}While Loop Condition:");
                self.print_expression(source, *condition, indent + 1);
                println!("{pad}While Body:");
                for &stmt in body { self.print_statement(source, stmt, indent + 1); }
            }
            Statement::Expression(expr_id) => {
                self.print_expression(source, *expr_id, indent);
            }
            _ => println!("{pad}Unknown/Unimplemented Statement Node"),
        }
    }

    fn print_expression(&self, source: &str, expr_id: ExprId, indent: usize) {
        let pad = "  ".repeat(indent);
        let expr = &self.expressions[expr_id.0 as usize];

        match expr {
            Expression::Literal { kind, .. } => {
                println!("{pad}Literal: {:?}", kind);
            }
            Expression::Identifier { span } => {
                let text = &source[span.start..span.end];
                println!("{pad}Identifier: {text}");
            }
            Expression::Binary { left, operator, right, .. } => {
                println!("{pad}BinaryOp: {:?}", operator);
                self.print_expression(source, *left, indent + 1);
                self.print_expression(source, *right, indent + 1);
            }
            Expression::Unary { operator, right, .. } => {
                println!("{pad}UnaryOp: {:?}", operator);
                self.print_expression(source, *right, indent + 1);
            }
            Expression::Call { callee, arguments, .. } => {
                println!("{pad}FunctionCall:");
                println!("{pad}  Callee:");
                self.print_expression(source, *callee, indent + 2);
                println!("{pad}  Arguments:");
                for &arg_id in arguments {
                    self.print_expression(source, arg_id, indent + 2);
                }
            }
        }
    }
        #[inline(always)]
    fn resolve_type_bytes(source: &str, type_span: &Span) -> Type {
        let type_bytes = unsafe { 
            source.as_bytes().get_unchecked(type_span.start as usize..type_span.end as usize) 
        };
        match type_bytes {
            b"Int64"   => Type::I64, b"Int32"   => Type::I32, b"Int16"   => Type::I16, b"Int8"    => Type::I8,
            b"Float64" => Type::F64, b"Float32" => Type::F32, b"Float16" => Type::F16, b"Float8"  => Type::F8,
            b"Uint64"  => Type::U64, b"Uint32"  => Type::U32, b"Uint16"  => Type::U16, b"Uint8"   => Type::U8,
            b"Bool"    => Type::Bool, b"String"  => Type::Str,
            _ => Type::Void, 
        }
    }

    pub fn prepare_caches(&mut self, source: &str, tokens: &[Token]) {
        self.resolved_expr_identifiers.resize(self.expressions.len(), IdentifierId(0));
        self.resolved_var_decl_types.resize(self.statements.len(), None);
        self.resolved_assignment_targets.resize(self.statements.len(), IdentifierId(0));
        self.resolved_function_names.resize(self.statements.len(), IdentifierId(0));
        self.resolved_function_params.resize(self.statements.len(), Vec::new());

        self.resolved_string_literals.clear(); 

        for (idx, expr) in self.expressions.iter().enumerate() {
            let expr_id = ExprId(idx as u32);
            
            match expr {
                Expression::Identifier { span } => {
                    if let Some(tok) = tokens.iter().find(|t| t.span.start == span.start) {
                        if let Some(pre_interned_id) = tok.id {
                            self.resolved_expr_identifiers[idx] = pre_interned_id;
                        }
                    }
                }
                Expression::Literal { kind: LiteralKind::String, span } => {
                    if let Some(tok) = tokens.iter().find(|t| t.span.start == span.start) {
                        if let Some(pre_interned_id) = tok.id {
                            self.resolved_string_literals.insert(expr_id, pre_interned_id);
                        }
                    }
                }
                _ => {}
            }
        }

        for (idx, stmt) in self.statements.iter().enumerate() {
            match stmt {
                Statement::VarDecl { name, explicit_type, .. } => {
                    if let Some(type_span) = explicit_type {
                        self.resolved_var_decl_types[idx] = Some(Self::resolve_type_bytes(source, type_span));
                    }
                    if let Some(tok) = tokens.iter().find(|t| t.span.start == name.start) {
                        if let Some(pre_interned_id) = tok.id {
                            self.resolved_assignment_targets[idx] = pre_interned_id;
                        }
                    }
                }
                Statement::Assignment { target, .. } => {
                    if let Some(tok) = tokens.iter().find(|t| t.span.start == target.start) {
                        if let Some(pre_interned_id) = tok.id {
                            self.resolved_assignment_targets[idx] = pre_interned_id;
                        }
                    }
                }
                Statement::Function { name, parameters, return_type, .. } => {
                    if let Some(type_span) = return_type {
                        self.resolved_var_decl_types[idx] = Some(Self::resolve_type_bytes(source, type_span));
                    }

                    if let Some(tok) = tokens.iter().find(|t| t.span.start == name.start) {
                        if let Some(pre_interned_id) = tok.id {
                            self.resolved_function_names[idx] = pre_interned_id;
                        }
                    }
                    
                    let mut cached_params = Vec::with_capacity(parameters.len());
                    for param in parameters {
                        let mut p_id = IdentifierId(0);
                        if let Some(tok) = tokens.iter().find(|t| t.span.start == param.name.start) {
                            if let Some(pre_interned_id) = tok.id {
                                p_id = pre_interned_id;
                            }
                        }
                        let p_ty = Self::resolve_type_bytes(source, &param.param_type);
                        cached_params.push((p_id, p_ty));
                    }
                    self.resolved_function_params[idx] = cached_params;
                }
                Statement::For { variable, .. } => {
                    if let Some(tok) = tokens.iter().find(|t| t.span.start == variable.start) {
                        if let Some(pre_interned_id) = tok.id {
                            self.resolved_assignment_targets[idx] = pre_interned_id;
                        }
                    }
                }
                Statement::Structure { name, fields, .. } => {
                    if let Some(tok) = tokens.iter().find(|t| t.span.start == name.start) {
                        if let Some(pre_interned_id) = tok.id {
                            self.resolved_function_names[idx] = pre_interned_id;
                        }
                    }
                    
                    let mut cached_fields = Vec::with_capacity(fields.len());
                    for field in fields {
                        let mut f_id = IdentifierId(0);
                        if let Some(tok) = tokens.iter().find(|t| t.span.start == field.name.start) {
                            if let Some(pre_interned_id) = tok.id {
                                f_id = pre_interned_id;
                            }
                        }
                        let f_ty = Self::resolve_type_bytes(source, &field.field_type);
                        cached_fields.push((f_id, f_ty));
                    }
                    self.resolved_function_params[idx] = cached_fields;
                }
                Statement::Connect { target, .. } => {
                    if let Some(tok) = tokens.iter().find(|t| t.span.start == target.start) {
                        if let Some(pre_interned_id) = tok.id {
                            self.resolved_assignment_targets[idx] = pre_interned_id;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            statements: Vec::with_capacity(1024),
            expressions: Vec::with_capacity(2024),
            root_statements: Vec::with_capacity(64),
            resolved_string_literals: HashMap::new(),
                
            resolved_expr_identifiers: Vec::new(),
            resolved_var_decl_types: Vec::new(),
            resolved_assignment_targets: Vec::new(),
            resolved_function_names: Vec::new(),
            resolved_function_params: Vec::new(),
        }
    }
    #[inline(always)]
    pub fn add_statement(&mut self, stmt: Statement) -> StmtId{
        let id = self.statements.len() as u32;
        self.statements.push(stmt);
        StmtId(id)
    }
    #[inline(always)]
    pub fn register_root(&mut self, id: StmtId) {
        self.root_statements.push(id);
    }
    #[inline(always)]
    pub fn add_expression(&mut self, expr: Expression) -> ExprId{
        let id = self.expressions.len() as u32;
        self.expressions.push(expr);
        ExprId(id)
    }
}