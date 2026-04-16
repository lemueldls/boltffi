use boltffi::*;

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn echo_bool(v: bool) -> bool {
    v
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn negate_bool(v: bool) -> bool {
    !v
}

#[export]
pub fn echo_i8(v: i8) -> i8 {
    v
}

#[export]
pub fn echo_u8(v: u8) -> u8 {
    v
}

#[export]
pub fn echo_i16(v: i16) -> i16 {
    v
}

#[export]
pub fn echo_u16(v: u16) -> u16 {
    v
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn echo_i32(v: i32) -> i32 {
    v
}

/// Adds two 32-bit signed integers and returns the result.
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn add_i32(a: i32, b: i32) -> i32 {
    a + b
}

#[export]
pub fn echo_u32(v: u32) -> u32 {
    v
}

#[export]
pub fn echo_i64(v: i64) -> i64 {
    v
}

#[export]
pub fn echo_u64(v: u64) -> u64 {
    v
}

#[export]
pub fn echo_f32(v: f32) -> f32 {
    v
}

#[export]
pub fn add_f32(a: f32, b: f32) -> f32 {
    a + b
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn echo_f64(v: f64) -> f64 {
    v
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn add_f64(a: f64, b: f64) -> f64 {
    a + b
}

#[export]
pub fn echo_usize(v: usize) -> usize {
    v
}

#[export]
pub fn echo_isize(v: isize) -> isize {
    v
}

/// A no-op call used to measure raw FFI overhead.
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn noop() {}

/// Benchmark alias that matches the existing benchmark harness naming.
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn add(a: i32, b: i32) -> i32 {
    add_i32(a, b)
}

/// Multiplies two double-precision numbers.
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn multiply(a: f64, b: f64) -> f64 {
    a * b
}
