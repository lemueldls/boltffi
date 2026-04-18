use crate::ir::{AbiContract, FfiContract};

use super::plan::KmpModule;

pub struct KmpLowerer<'a> {
    _ffi_contract: &'a FfiContract,
    _abi_contract: &'a AbiContract,
    package_name: String,
    module_name: String,
    library_name: String,
}

impl<'a> KmpLowerer<'a> {
    pub fn new(
        ffi_contract: &'a FfiContract,
        abi_contract: &'a AbiContract,
        package_name: String,
        module_name: String,
        library_name: String,
    ) -> Self {
        Self {
            _ffi_contract: ffi_contract,
            _abi_contract: abi_contract,
            package_name,
            module_name,
            library_name,
        }
    }

    pub fn lower(self) -> KmpModule {
        KmpModule {
            package_name: self.package_name,
            module_name: self.module_name,
            library_name: self.library_name,
        }
    }
}
