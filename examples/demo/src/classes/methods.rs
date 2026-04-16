use std::sync::Mutex;

#[cfg(not(feature = "uniffi"))]
use boltffi::*;

#[cfg(not(feature = "uniffi"))]
use crate::records::blittable::Point;

/// A simple thread-safe counter that demonstrates various
/// method return types: plain values, Option, Result, and
/// records.
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct Counter {
    count: Mutex<i32>,
}

#[cfg(not(feature = "uniffi"))]
#[export]
impl Counter {
    pub fn new(initial: i32) -> Counter {
        Counter {
            count: Mutex::new(initial),
        }
    }

    pub fn get(&self) -> i32 {
        *self.count.lock().unwrap()
    }

    pub fn increment(&self) {
        *self.count.lock().unwrap() += 1;
    }

    pub fn add(&self, amount: i32) {
        *self.count.lock().unwrap() += amount;
    }

    pub fn reset(&self) {
        *self.count.lock().unwrap() = 0;
    }

    /// Returns the current count if positive, or an error message.
    pub fn try_get_positive(&self) -> Result<i32, String> {
        let val = *self.count.lock().unwrap();
        if val > 0 {
            Ok(val)
        } else {
            Err("count is not positive".to_string())
        }
    }

    pub fn maybe_double(&self) -> Option<i32> {
        let val = *self.count.lock().unwrap();
        if val != 0 { Some(val * 2) } else { None }
    }

    pub fn as_point(&self) -> Point {
        Point {
            x: *self.count.lock().unwrap() as f64,
            y: 0.0,
        }
    }
}

#[cfg(feature = "uniffi")]
#[cfg_attr(feature = "uniffi", uniffi::export)]
impl Counter {
    #[uniffi::constructor]
    pub fn new(initial: i32) -> Counter {
        Counter {
            count: Mutex::new(initial),
        }
    }

    pub fn get(&self) -> i32 {
        *self.count.lock().unwrap()
    }

    pub fn increment(&self) {
        *self.count.lock().unwrap() += 1;
    }

    pub fn add(&self, amount: i32) {
        *self.count.lock().unwrap() += amount;
    }

    pub fn reset(&self) {
        *self.count.lock().unwrap() = 0;
    }
}
