use boltffi::*;
use std::collections::HashMap;

#[export]
pub fn echo_map_string_i32(v: HashMap<String, i32>) -> HashMap<String, i32> {
    v
}
