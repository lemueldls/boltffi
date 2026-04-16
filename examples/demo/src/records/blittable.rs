use boltffi::*;

/// A 2D point with double-precision coordinates.
#[data]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Point {
    /// Horizontal position.
    pub x: f64,
    /// Vertical position.
    pub y: f64,
}

#[data(impl)]
impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    pub fn origin() -> Self {
        Point { x: 0.0, y: 0.0 }
    }

    pub fn from_polar(r: f64, theta: f64) -> Self {
        Point {
            x: r * theta.cos(),
            y: r * theta.sin(),
        }
    }

    pub fn try_unit(x: f64, y: f64) -> Result<Self, String> {
        let len = (x * x + y * y).sqrt();
        if len == 0.0 {
            Err("cannot normalize zero vector".to_string())
        } else {
            Ok(Point {
                x: x / len,
                y: y / len,
            })
        }
    }

    pub fn checked_unit(x: f64, y: f64) -> Option<Self> {
        let len = (x * x + y * y).sqrt();
        if len == 0.0 {
            None
        } else {
            Some(Point {
                x: x / len,
                y: y / len,
            })
        }
    }

    pub fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn scale(&mut self, factor: f64) {
        self.x *= factor;
        self.y *= factor;
    }

    pub fn add(&self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    pub fn dimensions() -> u32 {
        2
    }
}

#[export]
pub fn echo_point(p: Point) -> Point {
    p
}

#[export]
pub fn try_make_point(x: f64, y: f64) -> Option<Point> {
    if x == 0.0 && y == 0.0 {
        None
    } else {
        Some(Point { x, y })
    }
}

#[export]
pub fn make_point(x: f64, y: f64) -> Point {
    Point { x, y }
}

#[export]
pub fn add_points(a: Point, b: Point) -> Point {
    Point {
        x: a.x + b.x,
        y: a.y + b.y,
    }
}

#[data]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[export]
pub fn echo_color(c: Color) -> Color {
    c
}

#[export]
pub fn make_color(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color { r, g, b, a }
}

/// A benchmark-friendly location record containing only primitive fields.
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[data]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Location {
    pub id: i64,
    pub lat: f64,
    pub lng: f64,
    pub rating: f64,
    pub review_count: i32,
    pub is_open: bool,
}

/// A benchmark-friendly trade record with dense numeric fields.
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[data]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Trade {
    pub id: i64,
    pub symbol_id: i32,
    pub price: f64,
    pub quantity: i64,
    pub bid: f64,
    pub ask: f64,
    pub volume: i64,
    pub timestamp: i64,
    pub is_buy: bool,
}

/// A densely packed physics particle used for payload-heavy benchmarks.
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[data]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Particle {
    pub id: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub mass: f64,
    pub charge: f64,
    pub active: bool,
}

/// A dense sensor record used for structured benchmark payloads.
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[data]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct SensorReading {
    pub sensor_id: i64,
    pub timestamp: i64,
    pub temperature: f64,
    pub humidity: f64,
    pub pressure: f64,
    pub light: f64,
    pub battery: f64,
    pub signal_strength: i32,
    pub is_valid: bool,
}

/// A timestamped data point used by callback and object benchmarks.
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[data]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
    pub timestamp: i64,
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn generate_locations(count: i32) -> Vec<Location> {
    (0..count)
        .map(|index| Location {
            id: i64::from(index),
            lat: 37.7749 + f64::from(index) * 0.001,
            lng: -122.4194 + f64::from(index) * 0.001,
            rating: 3.0 + f64::from(index % 20) * 0.1,
            review_count: 10 + index * 5,
            is_open: index % 2 == 0,
        })
        .collect()
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn process_locations(locations: Vec<Location>) -> i32 {
    locations.len() as i32
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn sum_ratings(locations: Vec<Location>) -> f64 {
    locations.iter().map(|location| location.rating).sum()
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn generate_trades(count: i32) -> Vec<Trade> {
    (0..count)
        .map(|index| Trade {
            id: i64::from(index),
            symbol_id: index % 500,
            price: 100.0 + f64::from(index) * 0.01,
            quantity: i64::from(index % 1000) + 1,
            bid: 99.95 + f64::from(index) * 0.01,
            ask: 100.05 + f64::from(index) * 0.01,
            volume: i64::from(index) * 1000,
            timestamp: 1_700_000_000_000 + i64::from(index) * 1000,
            is_buy: index % 2 == 0,
        })
        .collect()
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn sum_trade_volumes(trades: Vec<Trade>) -> i64 {
    trades.iter().map(|trade| trade.volume).sum()
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn generate_particles(count: i32) -> Vec<Particle> {
    (0..count)
        .map(|index| Particle {
            id: i64::from(index),
            x: f64::from(index) * 0.1,
            y: f64::from(index) * 0.2,
            z: f64::from(index) * 0.3,
            vx: f64::from(index) * 0.01,
            vy: f64::from(index) * 0.02,
            vz: f64::from(index) * 0.03,
            mass: 1.0 + f64::from(index) * 0.001,
            charge: if index % 2 == 0 { 1.0 } else { -1.0 },
            active: index % 10 != 0,
        })
        .collect()
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn sum_particle_masses(particles: Vec<Particle>) -> f64 {
    particles.iter().map(|particle| particle.mass).sum()
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn generate_sensor_readings(count: i32) -> Vec<SensorReading> {
    (0..count)
        .map(|index| SensorReading {
            sensor_id: i64::from(index % 100),
            timestamp: 1_700_000_000_000 + i64::from(index) * 100,
            temperature: 20.0 + f64::from(index % 30),
            humidity: 40.0 + f64::from(index % 40),
            pressure: 1_013.25 + f64::from(index % 20),
            light: f64::from(index % 1000),
            battery: 100.0 - f64::from(index % 100),
            signal_strength: -50 - (index % 50),
            is_valid: index % 20 != 0,
        })
        .collect()
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn avg_sensor_temperature(readings: Vec<SensorReading>) -> f64 {
    let count = readings.len();
    if count == 0 {
        0.0
    } else {
        readings
            .iter()
            .map(|reading| reading.temperature)
            .sum::<f64>()
            / count as f64
    }
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn find_location(id: i32) -> Option<Location> {
    if id > 0 {
        Some(Location {
            id: i64::from(id),
            lat: 37.7749,
            lng: -122.4194,
            rating: 4.5,
            review_count: 100,
            is_open: true,
        })
    } else {
        None
    }
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
#[export]
pub fn find_locations(count: i32) -> Option<Vec<Location>> {
    if count > 0 {
        Some(generate_locations(count))
    } else {
        None
    }
}
