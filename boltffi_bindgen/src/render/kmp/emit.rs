use super::plan::{KmpModule, KmpOutputs};
use super::templates::KmpTemplates;

pub struct KmpEmitter;

impl KmpEmitter {
    pub fn emit(module: &KmpModule) -> KmpOutputs {
        KmpTemplates::render(module)
    }
}
