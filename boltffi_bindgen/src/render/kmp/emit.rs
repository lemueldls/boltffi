use super::lower::KmpLowerer;
use super::plan::{KmpOptions, KmpOutputs};
use super::templates;
use crate::ir::{AbiContract, contract::FfiContract};

pub struct KmpEmitter;

impl KmpEmitter {
    pub fn emit(contract: &FfiContract, abi: &AbiContract, options: &KmpOptions) -> KmpOutputs {
        let module = KmpLowerer::new(contract, abi).lower();
        templates::render_outputs(&module, options)
    }
}
