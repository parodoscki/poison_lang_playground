use crate::llvm::main_codegen::llvm_codegen::CodegenEnv;
use llvm_sys::{core::*, prelude::LLVMValueRef}; 
use crate::llvm::main_codegen::llvm_codegen::LLVMBackend;
use std::{collections::HashMap, ffi::CString};
use compiler_frontend::{poison_abstract_syntax_tree_nodes::*, poison_semantic_analyzer::poison_symbol_table::*};
pub struct LLVMWalkerCodeGenerator {
    pub backend: LLVMBackend,
    pub env: CodegenEnv,
    pub global_functions: HashMap<IdentifierId, LLVMValueRef>,
}
impl LLVMWalkerCodeGenerator {
    pub fn new() -> Self {
        Self {
            backend: LLVMBackend::new(),
            env: CodegenEnv::new(),
            global_functions: HashMap::new(),
        }
    }
    #[inline(always)]
    fn make_expression(&self, id: u32, left: LLVMValueRef, right: LLVMValueRef, is_float: bool) -> LLVMValueRef {
        unsafe {
            if is_float {
                match id {
                    0 => LLVMBuildFAdd(self.backend.builder, left, right, b"fadd_tmp\0".as_ptr() as *const _),
                    1 => LLVMBuildFSub(self.backend.builder, left, right, b"fsub_tmp\0".as_ptr() as *const _),
                    2 => LLVMBuildFMul(self.backend.builder, left, right, b"fmul_tmp\0".as_ptr() as *const _),
                    3 => LLVMBuildFDiv(self.backend.builder, left, right, b"fdiv_tmp\0".as_ptr() as *const _),
                    _ => std::ptr::null_mut()
                }
            } else {
                match id {
                    0 => LLVMBuildAdd(self.backend.builder, left, right, b"add_tmp\0".as_ptr() as *const _),
                    1 => LLVMBuildSub(self.backend.builder, left, right, b"sub_tmp\0".as_ptr() as *const _),
                    2 => LLVMBuildMul(self.backend.builder, left, right, b"mul_tmp\0".as_ptr() as *const _),
                    3 => LLVMBuildSDiv(self.backend.builder, left, right, b"div_tmp\0".as_ptr() as *const _),
                    _ => std::ptr::null_mut()
                }
            }
        }
    }
    pub fn walk_expression(&mut self, ast: &Ast, expr_id: ExprId, interner: &Interner) -> LLVMValueRef{
        unsafe {
            let expr = &ast.expressions[expr_id.0 as usize];
            match expr {
                Expression::Literal { kind, .. } => {
                    match kind {
                        LiteralKind::Integer(val) => {
                            let i32_ty = self.backend.map_id_to_type(2); 
                            LLVMConstInt(i32_ty, *val as u64, 1)
                        }
                        LiteralKind::Float(val) => {
                    
                            let f32_ty = self.backend.map_id_to_type(6);
                            LLVMConstReal(f32_ty, *val)
                        }
                        LiteralKind::Boolean(val) => {
                            let i8_ty = self.backend.map_id_to_type(4);
                            LLVMConstInt(i8_ty, if *val { 1 } else { 0 }, 0)
                        }
                        LiteralKind::String => {
                            let string_id = ast.resolved_string_literals.get(&expr_id)
                            .expect("Codegen Error: Unresolved string literal metadata index.");

                            let string_val = interner.lookup(*string_id).unwrap_or("");
                            
                            let c_str = CString::new(string_val).unwrap_or_else(|_| CString::new("").unwrap());
                            
                            llvm_sys::core::LLVMBuildGlobalStringPtr(
                                self.backend.builder,
                                c_str.as_ptr(),
                                b"str_lit\0".as_ptr() as *const _,
                            )
                        },
                    }
                },
                Expression::Identifier { .. } => {
                    let name_id = *ast.resolved_expr_identifiers.get_unchecked(expr_id.0 as usize);
                    
                    let var_ptr = self.env.lookup(name_id).expect("Codegen Error: Undefined variable lookup.");
                    
                    let var_type = LLVMGetAllocatedType(var_ptr);
                    LLVMBuildLoad2(self.backend.builder, var_type, var_ptr, b"load_tmp\0".as_ptr() as *const _)
                },
                Expression::Binary { left, operator, right, span } => {
                    let left_val = self.walk_expression(ast, *left,interner);
                    let right_val = self.walk_expression(ast, *right,interner);

                    match operator {
                        OpKind::Equal | OpKind::NotEqual | OpKind::Less | 
                        OpKind::LessEqual | OpKind::Greater | OpKind::GreaterEqual => {
                            
                            let predicate = match operator {
                                OpKind::Equal        => llvm_sys::LLVMIntPredicate::LLVMIntEQ,
                                OpKind::NotEqual     => llvm_sys::LLVMIntPredicate::LLVMIntNE,
                                OpKind::Less         => llvm_sys::LLVMIntPredicate::LLVMIntSLT, // Signed Less Than
                                OpKind::LessEqual    => llvm_sys::LLVMIntPredicate::LLVMIntSLE, // Signed Less or Equal
                                OpKind::Greater      => llvm_sys::LLVMIntPredicate::LLVMIntSGT, // Signed Greater Than
                                OpKind::GreaterEqual => llvm_sys::LLVMIntPredicate::LLVMIntSGE, // Signed Greater or Equal
                                _ => unreachable!(),
                            };

                            return LLVMBuildICmp(
                                self.backend.builder,
                                predicate,
                                left_val,
                                right_val,
                                b"cmp_tmp\0".as_ptr() as *const _,
                            );
                        }
                        _ => {}
                    }

                    let op_id = match operator {
                        OpKind::Add => 0,
                        OpKind::Sub => 1,
                        OpKind::Mul => 2,
                        OpKind::Div => 3,
                        _ => 0,
                    };

                    let is_float = false;

                    self.make_expression(op_id, left_val, right_val, is_float)
                },
                Expression::Unary { operator, right, .. } => {
                    let right_val = self.walk_expression(ast, *right, interner);
                    match operator {
                        OpKind::Sub => {
                            let zero = LLVMConstInt(LLVMTypeOf(right_val), 0, 1);
                            LLVMBuildSub(self.backend.builder, zero, right_val, b"neg_tmp\0".as_ptr() as *const _)
                        }
                        _ => right_val,
                    }
                },
                Expression::Call { callee, arguments , .. } => {
                    let callee_expr = ast.expressions.get_unchecked(callee.0 as usize);
                    
                    let func_ptr = match callee_expr {
                        Expression::Identifier { .. } => {
                            let func_name_id = *ast.resolved_expr_identifiers.get_unchecked(callee.0 as usize);
                            
                            // FIX: Pull directly from the global functions registry instead of stack-based env
                            *self.global_functions.get(&func_name_id)
                                .expect("Codegen Error: Attempted to call an undefined function.")
                        }
                        _ => panic!("Codegen Error: Complex or dynamic function pointers are not supported yet.")
                    };

                    assert!(!func_ptr.is_null(), "Codegen Error: Target function pointer is null!");

                    // Safely extract the function type signature from the global value pointer
                    let func_type = unsafe { llvm_sys::core::LLVMGlobalGetValueType(func_ptr) };
                    assert!(!func_type.is_null(), "Codegen Error: Failed to extract function type signature.");

                    let mut llvm_args = [std::ptr::null_mut(); 16];
                    let arg_count = arguments.len();

                    if arg_count > 16 {
                        panic!("Codegen Error: Function call passes {} arguments, exceeding the 16-parameter limit.", arg_count);
                    }

                    for (i, &arg_id) in arguments.iter().enumerate() {
                        llvm_args[i] = self.walk_expression(ast, arg_id, interner);
                    }

                    // Pass std::ptr::null_mut() safely if there are zero arguments to avoid slice-pointer faults
                    let arg_ptr = if arg_count == 0 { std::ptr::null_mut() } else { llvm_args.as_mut_ptr() };

                    unsafe {
                        LLVMBuildCall2(
                            self.backend.builder,
                            func_type,
                            func_ptr,
                            arg_ptr,
                            arg_count as u32,
                            b"call_tmp\0".as_ptr() as *const _,
                        )
                    }
                },

            }
        }
    }
    pub unsafe fn declare_printf(&mut self) -> llvm_sys::prelude::LLVMValueRef {        
        let context = self.backend.context;
        let module = self.backend.module;

        let return_type = llvm_sys::core::LLVMInt32TypeInContext(context);

        let i8_ty = llvm_sys::core::LLVMInt8TypeInContext(context);
        let mut param_types = [llvm_sys::core::LLVMPointerType(i8_ty, 0)];

        let printf_type = llvm_sys::core::LLVMFunctionType(
            return_type,
            param_types.as_mut_ptr(),
            1,
            1,
        );

        let printf_func = llvm_sys::core::LLVMAddFunction(
            module,
            b"printf\0".as_ptr() as *const _,
            printf_type,
        );

        llvm_sys::core::LLVMSetFunctionCallConv(printf_func, llvm_sys::LLVMCallConv::LLVMCCallConv as u32);

        printf_func
    }
    pub fn walk_statement(&mut self, ast: &Ast, stmt_id: StmtId, interner: &Interner) {
        unsafe {
            let stmt = &ast.statements[stmt_id.0 as usize];
            match stmt {
                Statement::If { condition, then_branch, else_branch, span } => unsafe {
                    let current_func = self.backend.get_current_function();

                    let check_if_inside_func = current_func.as_ref().expect("Codegen Error: 'if' statement declared outside of a valid function body.");
                    let cond_value = self.walk_expression(ast, *condition, interner);
                    let cond_bool = LLVMBuildICmp(
                        self.backend.builder,
                        llvm_sys::LLVMIntPredicate::LLVMIntNE,
                        cond_value,
                        LLVMConstInt(LLVMTypeOf(cond_value), 0, 0),
                        b"if_cond_bit\0".as_ptr() as *const _,
                    );

                    let then_block = self.backend.ctrl_emitter.append_block(self.backend.context, current_func, "then_branch");
                    let else_block = self.backend.ctrl_emitter.append_block(self.backend.context, current_func, "else_branch");
                    let merge_block = self.backend.ctrl_emitter.append_block(self.backend.context, current_func, "if_merge");

                    LLVMBuildCondBr(self.backend.builder, cond_bool, then_block, else_block);

                    LLVMPositionBuilderAtEnd(self.backend.builder, then_block);
                    self.env.enter_scope();
                    for &inner_stmt_id in then_branch {
                        self.walk_statement(ast, inner_stmt_id, interner);
                    }
                    self.env.exit_scope();
                    if LLVMGetInsertBlock(self.backend.builder) != std::ptr::null_mut() {
                        LLVMBuildBr(self.backend.builder, merge_block);
                    }

                    LLVMPositionBuilderAtEnd(self.backend.builder, else_block);
                    self.env.enter_scope();
                    if let Some(else_stmts) = else_branch {
                        for &inner_stmt_id in else_stmts {
                            self.walk_statement(ast, inner_stmt_id, interner);
                        }
                    }
                    self.env.exit_scope();
                    if LLVMGetInsertBlock(self.backend.builder) != std::ptr::null_mut() {
                        LLVMBuildBr(self.backend.builder, merge_block);
                    }

                    LLVMPositionBuilderAtEnd(self.backend.builder, merge_block);
                }
                Statement::VarDecl {initializer, ..} => {
                    let initial_value = self.walk_expression(ast, *initializer, interner);
                    let expected_ty = ast.resolved_var_decl_types.get_unchecked(stmt_id.0 as usize).unwrap_or(Type::I32);
                    let name_id = *ast.resolved_assignment_targets.get_unchecked(stmt_id.0 as usize);

                    let llvm_ty = self.backend.map_id_to_type(expected_ty.to_backend_id());
                    let var_name = interner.lookup(name_id).unwrap_or("unnamed_var");

                    let c_var_name = CString::new(var_name).unwrap_or_else(|_| CString::new("unnamed_var").unwrap());

                    let var_ptr = self.backend.var_emitter.make_variable(
                        self.backend.builder, 
                        llvm_ty, 
                        c_var_name.as_ptr(),
                        initial_value
                    );
                    
                    
                    self.env.insert(name_id, var_ptr);
                }
                Statement::While { condition, body, span } => unsafe {
                    let current_func = self.backend.get_current_function();
                    
                    let check_if_inside_func = current_func.as_ref()
                                .expect("Codegen Error: 'while' loop declared outside of a valid function body.");

                    let loop_cond_block = self.backend.ctrl_emitter.append_block(self.backend.context, current_func, "while_cond");
                    let loop_body_block = self.backend.ctrl_emitter.append_block(self.backend.context, current_func, "while_body");
                    let loop_end_block = self.backend.ctrl_emitter.append_block(self.backend.context, current_func, "while_end");

                    LLVMBuildBr(self.backend.builder, loop_cond_block);

                    LLVMPositionBuilderAtEnd(self.backend.builder, loop_cond_block);
                    let cond_value = self.walk_expression(ast, *condition, interner);
                    let cond_boolean = LLVMBuildICmp(
                        self.backend.builder, 
                        llvm_sys::LLVMIntPredicate::LLVMIntNE, 
                        cond_value, 
                        LLVMConstInt(LLVMTypeOf(cond_value), 0, 0), 
                        b"w_bit\0".as_ptr() as *const _
                    );
                    LLVMBuildCondBr(self.backend.builder, cond_boolean, loop_body_block, loop_end_block);

                    LLVMPositionBuilderAtEnd(self.backend.builder, loop_body_block);
                    self.env.enter_scope();
                    for &inner_stmt_id in body {
                        self.walk_statement(ast, inner_stmt_id, interner);
                    }
                    self.env.exit_scope();
                    
                    if LLVMGetInsertBlock(self.backend.builder) != std::ptr::null_mut() {
                        LLVMBuildBr(self.backend.builder, loop_cond_block);
                    }

                    LLVMPositionBuilderAtEnd(self.backend.builder, loop_end_block);
                }

                Statement::Assignment { target, operator, value, span } => {
                    let new_value = self.walk_expression(ast, *value, interner);

                    let target_name_id = ast.resolved_assignment_targets[stmt_id.0 as usize];

                    let var_ptr = self.env.lookup(target_name_id).unwrap_or_else(|| {
                        panic!("Poison.CodegenError: Attempted to assign to an undefined variable.");
                    });

                    match operator {
                        OpKind::Assign => {
                            LLVMBuildStore(self.backend.builder, new_value, var_ptr);
                        }
                        _ => {
                            panic!("Poison.CodegenError: Compound assignments are not implemented yet.");
                        }
                    }
                }
                Statement::Function { body, .. } => {
                    let func_name_id = ast.resolved_function_names
                        .get(stmt_id.0 as usize)
                        .copied()
                        .expect("Poison.CodegenError: Missing function name ID");

                    let params_meta = ast.resolved_function_params
                        .get(stmt_id.0 as usize)
                        .expect("Poison.CodegenError: Missing function parameters metadata");
                    let func_name = interner.lookup(func_name_id).unwrap_or("unnamed_func");
                    
                    let mut param_types = [std::ptr::null_mut(); 64];
                    let mut param_names = [""; 64]; 

                    if params_meta.len() > 64 {
                        panic!("Poison.CodegenError: Function '{}' exceeds the 64 parameter limit.", func_name);
                    }

                    for (i, &(param_id, ty)) in params_meta.iter().enumerate() {
                        param_types[i] = self.backend.map_id_to_type(ty.to_backend_id());
                        param_names[i] = interner.lookup(param_id).unwrap_or("param");
                    }

                    let return_type: Type = ast.resolved_var_decl_types
                        .get(stmt_id.0 as usize)
                        .and_then(|inner_option| inner_option.clone()) // Extracts the inner Option<Type>
                        .unwrap_or(Type::Void);

                    let llvm_ret_ty: *mut llvm_sys::LLVMType = self.backend.map_id_to_type(return_type.to_backend_id());

                    let (function, entry_block) = self.backend.func_emitter.add_function(
                        self.backend.module,
                        self.backend.context,
                        self.backend.builder,
                        llvm_ret_ty,
                        &mut param_types[..params_meta.len()],
                        &param_names[..params_meta.len()],
                        func_name
                    );

                    // Make sure the builder is properly placed at the entry block
                    LLVMPositionBuilderAtEnd(self.backend.builder, entry_block);


                    // This leaves the function registered in the global scope frame forever.
                    self.global_functions.insert(func_name_id, function);
                    
                    self.env.enter_scope();
                    
                    let c_param_names: Vec<CString> = param_names[..params_meta.len()]
                        .iter()
                        .map(|&p_name| CString::new(p_name).unwrap_or_else(|_| CString::new("param").unwrap()))
                        .collect();

                    for (i, &(param_id, ty)) in params_meta.iter().enumerate() {
                        let llvm_param_val = LLVMGetParam(function, i as u32);
                        let llvm_param_ty = param_types[i];
                        
                        let var_ptr = self.backend.var_emitter.make_variable(
                            self.backend.builder,
                            llvm_param_ty,
                            c_param_names[i].as_ptr(),
                            llvm_param_val
                        );
                        
                        self.env.insert(param_id, var_ptr);
                    }

                    for &inner_stmt_id in body {
                        self.walk_statement(ast, inner_stmt_id, interner); 
                    }
                    
                    drop(c_param_names);
                    self.env.exit_scope();
                }


                Statement::Return{value, ..} => {
                    if let Some(expr_id) = value {
                        let return_val = self.walk_expression(ast, *expr_id, interner);
                        
                        LLVMBuildRet(self.backend.builder, return_val);
                    } else {
                        LLVMBuildRetVoid(self.backend.builder);
                    }
                }
                Statement::Expression(expr_id) => {
                    self.walk_expression(ast, *expr_id, interner);
                }

                _ => {}
            }
        }
    }
}
