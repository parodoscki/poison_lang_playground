use llvm_sys::core::{LLVMBuildAlloca, LLVMBuildStore};
use llvm_sys::prelude::{LLVMBuilderRef, LLVMTypeRef, LLVMValueRef};
use std::os::raw::c_char;

pub struct LLVMVariableEmitter;

impl LLVMVariableEmitter {
    pub unsafe fn make_variable(
        &self, 
        builder: LLVMBuilderRef, 
        var_type: LLVMTypeRef, 
        name_ptr: *const c_char, 
        value: LLVMValueRef
    ) -> LLVMValueRef {
        let var_ptr = LLVMBuildAlloca(builder, var_type, name_ptr);
        LLVMBuildStore(builder, value, var_ptr);
        var_ptr
    }
}
