use std::ffi::CString;
use llvm_sys::core::{
    LLVMAddFunction, LLVMAppendBasicBlockInContext, LLVMBuildRet, LLVMBuildRetVoid,
    LLVMFunctionType, LLVMGetParam, LLVMPositionBuilderAtEnd, LLVMSetValueName2,
};
use llvm_sys::prelude::{
    LLVMBuilderRef, LLVMContextRef, LLVMBasicBlockRef, LLVMModuleRef, LLVMTypeRef, LLVMValueRef,
};
pub struct LLVMFunctionEmitter;
impl LLVMFunctionEmitter {
    pub unsafe fn add_function(
        &self, 
        module: LLVMModuleRef, 
        context: LLVMContextRef, 
        builder: LLVMBuilderRef,
        return_type: LLVMTypeRef, 
        param_types: &mut [LLVMTypeRef], 
        param_names: &[&str], 
        name: &str
    ) -> (LLVMValueRef, LLVMBasicBlockRef) {

        let param_ptr = if param_types.is_empty() {
            std::ptr::null_mut()
        } else {
            param_types.as_mut_ptr()
        };

        let func_type = LLVMFunctionType(return_type, param_ptr, param_types.len() as u32, 0);
        
        let c_name = CString::new(name).unwrap();
        let function = LLVMAddFunction(module, c_name.as_ptr(), func_type);
        assert!(!function.is_null(), "LLVMAddFunction returned null for '{}'", name);

        let c_entry = CString::new("entry").unwrap();
        let entry_block = LLVMAppendBasicBlockInContext(context, function, c_entry.as_ptr());
        assert!(!entry_block.is_null(), "LLVMAppendBasicBlockInContext returned null for '{}'", name);

        let c_param_names: Vec<CString> = param_names
            .iter()
            .map(|&p_name| CString::new(p_name).unwrap())
            .collect();

        for (i, c_pname) in c_param_names.iter().enumerate() {
            let param_value = LLVMGetParam(function, i as u32);
            if !param_value.is_null() {
                LLVMSetValueName2(param_value, c_pname.as_ptr(), param_names[i].len());
            }
        }
        (function, entry_block)
    }

    pub fn return_value(&self, value: Option<LLVMValueRef>, builder: LLVMBuilderRef) -> LLVMValueRef {
        unsafe {
            match value {
                Some(return_val) => {
                    LLVMBuildRet(builder, return_val)
                }
                None => {
                    LLVMBuildRetVoid(builder)
                }
            }
        }
    }
}
