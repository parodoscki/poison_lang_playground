use std::ffi::{CStr, CString};
use llvm_sys::core::{
    LLVMContextCreate, LLVMContextDispose, LLVMCreateBuilderInContext, LLVMDisposeBuilder,
    LLVMDisposeModule, LLVMDumpModule, LLVMGetBasicBlockParent, LLVMGetInsertBlock,
    LLVMInt16TypeInContext, LLVMInt32TypeInContext, LLVMInt64TypeInContext, LLVMInt8TypeInContext,
    LLVMModuleCreateWithNameInContext, LLVMVoidTypeInContext, LLVMDoubleTypeInContext,
    LLVMFloatTypeInContext, LLVMHalfTypeInContext, LLVMPrintModuleToString,
};
use llvm_sys::prelude::{LLVMBuilderRef, LLVMContextRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef};

use crate::llvm::emitters::llvm_control_flow_emitter::LLVMControlFlowEmitter;
use crate::llvm::emitters::llvm_function_emitter::LLVMFunctionEmitter;
use crate::llvm::emitters::llvm_variable_emitter::LLVMVariableEmitter;
use compiler_frontend::poison_semantic_analyzer::poison_symbol_table::*;


pub struct LLVMBackend {
    pub context: LLVMContextRef,
    pub module: LLVMModuleRef,
    pub builder: LLVMBuilderRef,

    pub var_emitter: LLVMVariableEmitter,
    pub func_emitter: LLVMFunctionEmitter,
    pub ctrl_emitter: LLVMControlFlowEmitter,
}
impl LLVMBackend {
    pub fn new() -> Self {
        unsafe {
            let context = LLVMContextCreate();

            let module_name = CString::new("my_module").unwrap();
            let module = LLVMModuleCreateWithNameInContext(module_name.as_ptr(), context);
                
            let builder = LLVMCreateBuilderInContext(context);

            Self {
                context,
                module,
                builder,
                var_emitter: LLVMVariableEmitter,
                func_emitter: LLVMFunctionEmitter,
                ctrl_emitter: LLVMControlFlowEmitter,
            }
        }
    }
    #[inline(always)]
    pub unsafe fn get_current_function(&self) -> LLVMValueRef {
        let current_block = LLVMGetInsertBlock(self.builder);
        LLVMGetBasicBlockParent(current_block)
    }
    // id 0 = void_type

    // id 1 = i64
    // id 2 = i32
    // id 3 = i16
    // id 4 = i8

    // id 5 = f64
    // id 6 = f32
    // id 7 = f16
    // id 8 = f8

    pub unsafe fn map_id_to_type(&self, id: u32) -> LLVMTypeRef {
        match id {
            0 => LLVMVoidTypeInContext(self.context),
            1 => LLVMInt64TypeInContext(self.context),
            2 => LLVMInt32TypeInContext(self.context),
            3 => LLVMInt16TypeInContext(self.context),
            4 => LLVMInt8TypeInContext(self.context),
            5 => LLVMDoubleTypeInContext(self.context),
            6 => LLVMFloatTypeInContext(self.context),
            7 => LLVMHalfTypeInContext(self.context),
            9 => {
                let i8_ty = llvm_sys::core::LLVMInt8TypeInContext(self.context);
                
                // If you are targeting LLVM 15+ (Opaque pointers), you can use LLVMPointerTypeInContext
                // For standard typed pointers, map it as a pointer to the i8 base type:
                llvm_sys::core::LLVMPointerType(i8_ty, 0) 
            }
            _ => panic!("Unknown type ID: {}", id),
        }
    }
    pub unsafe fn get_llvm_ir_string(&mut self) -> String {
        let raw_c_str = LLVMPrintModuleToString(self.module);
        
        if raw_c_str.is_null() {
            return String::from("; Error: Failed to generate LLVM IR");
        }
        let rust_str = CStr::from_ptr(raw_c_str).to_string_lossy().into_owned();
        
        llvm_sys::core::LLVMDisposeMessage(raw_c_str);
        
        rust_str
    }
    /*
        id 0 = add
        id 1 = sub
        id 2 = mul
        id 3 = div

        id 4 = function call
    */
    pub fn dump(&self) {
        unsafe {
            LLVMDumpModule(self.module);
        }
    }
    pub fn dispose(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
            LLVMContextDispose(self.context);
        }
    }
}

pub struct CodegenEnv {
    bindings: Vec<(IdentifierId, LLVMValueRef)>,
    scope_markers: Vec<usize>,
}

impl CodegenEnv {
    pub fn new() -> Self {
        Self { bindings: Vec::new(), scope_markers: Vec::new() }
    }
    pub fn enter_scope(&mut self) { self.scope_markers.push(self.bindings.len()); }
    pub fn exit_scope(&mut self) {
        if let Some(prev_len) = self.scope_markers.pop() { self.bindings.truncate(prev_len); }
    }
    pub fn insert(&mut self, id: IdentifierId, val: LLVMValueRef) { self.bindings.push((id, val)); }
    pub fn lookup(&self, id: IdentifierId) -> Option<LLVMValueRef> {
        for &(bind_id, ptr) in self.bindings.iter().rev() {
            if bind_id == id { return Some(ptr); }
        }
        None
    }
}
