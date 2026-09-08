use std::ffi::CString;
use llvm_sys::core::LLVMAppendBasicBlockInContext;
use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMContextRef, LLVMValueRef};

pub struct LLVMControlFlowEmitter;
impl LLVMControlFlowEmitter {
    pub unsafe fn append_block(&self, context: LLVMContextRef, function: LLVMValueRef, name: &str) -> LLVMBasicBlockRef {
        if function.is_null() {
            panic!(
                "CRITICAL CODEGEN ERROR: Attempted to append a basic block named '{}' to a NULL function pointer!\n\
                 Ensure your AST walker has explicitly positioned the builder inside a valid function definition.",
                name
            );
        }
        let c_name = CString::new(name).unwrap();
        let block_ptr = LLVMAppendBasicBlockInContext(context, function, c_name.as_ptr());
        
        drop(c_name); 

        block_ptr
    }
}