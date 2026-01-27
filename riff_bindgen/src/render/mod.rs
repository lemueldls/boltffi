pub mod swift;
pub mod jni;

use crate::ir::{AbiContract, FfiContract};

pub trait Renderer {
    type Output;

    fn render(contract: &FfiContract, abi: &AbiContract) -> Self::Output;
}
