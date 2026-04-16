use demo as demo_api;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
pub struct Location {
    pub id: i64,
    pub lat: f64,
    pub lng: f64,
    pub rating: f64,
    pub review_count: i32,
    pub is_open: bool,
}

#[wasm_bindgen]
impl Location {
    #[wasm_bindgen(constructor)]
    pub fn new(id: i64, lat: f64, lng: f64, rating: f64, review_count: i32, is_open: bool) -> Self {
        Self {
            id,
            lat,
            lng,
            rating,
            review_count,
            is_open,
        }
    }
}

impl From<demo_api::Location> for Location {
    fn from(location: demo_api::Location) -> Self {
        Self {
            id: location.id,
            lat: location.lat,
            lng: location.lng,
            rating: location.rating,
            review_count: location.review_count,
            is_open: location.is_open,
        }
    }
}

impl From<Location> for demo_api::Location {
    fn from(location: Location) -> Self {
        Self {
            id: location.id,
            lat: location.lat,
            lng: location.lng,
            rating: location.rating,
            review_count: location.review_count,
            is_open: location.is_open,
        }
    }
}

#[wasm_bindgen]
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

#[wasm_bindgen]
impl Trade {
    #[wasm_bindgen(constructor)]
    pub fn new(
        id: i64,
        symbol_id: i32,
        price: f64,
        quantity: i64,
        bid: f64,
        ask: f64,
        volume: i64,
        timestamp: i64,
        is_buy: bool,
    ) -> Self {
        Self {
            id,
            symbol_id,
            price,
            quantity,
            bid,
            ask,
            volume,
            timestamp,
            is_buy,
        }
    }
}

impl From<demo_api::Trade> for Trade {
    fn from(trade: demo_api::Trade) -> Self {
        Self {
            id: trade.id,
            symbol_id: trade.symbol_id,
            price: trade.price,
            quantity: trade.quantity,
            bid: trade.bid,
            ask: trade.ask,
            volume: trade.volume,
            timestamp: trade.timestamp,
            is_buy: trade.is_buy,
        }
    }
}

impl From<Trade> for demo_api::Trade {
    fn from(trade: Trade) -> Self {
        Self {
            id: trade.id,
            symbol_id: trade.symbol_id,
            price: trade.price,
            quantity: trade.quantity,
            bid: trade.bid,
            ask: trade.ask,
            volume: trade.volume,
            timestamp: trade.timestamp,
            is_buy: trade.is_buy,
        }
    }
}

#[wasm_bindgen]
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

#[wasm_bindgen]
impl Particle {
    #[wasm_bindgen(constructor)]
    pub fn new(
        id: i64,
        x: f64,
        y: f64,
        z: f64,
        vx: f64,
        vy: f64,
        vz: f64,
        mass: f64,
        charge: f64,
        active: bool,
    ) -> Self {
        Self {
            id,
            x,
            y,
            z,
            vx,
            vy,
            vz,
            mass,
            charge,
            active,
        }
    }
}

impl From<demo_api::Particle> for Particle {
    fn from(particle: demo_api::Particle) -> Self {
        Self {
            id: particle.id,
            x: particle.x,
            y: particle.y,
            z: particle.z,
            vx: particle.vx,
            vy: particle.vy,
            vz: particle.vz,
            mass: particle.mass,
            charge: particle.charge,
            active: particle.active,
        }
    }
}

impl From<Particle> for demo_api::Particle {
    fn from(particle: Particle) -> Self {
        Self {
            id: particle.id,
            x: particle.x,
            y: particle.y,
            z: particle.z,
            vx: particle.vx,
            vy: particle.vy,
            vz: particle.vz,
            mass: particle.mass,
            charge: particle.charge,
            active: particle.active,
        }
    }
}

#[wasm_bindgen]
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

#[wasm_bindgen]
impl SensorReading {
    #[wasm_bindgen(constructor)]
    pub fn new(
        sensor_id: i64,
        timestamp: i64,
        temperature: f64,
        humidity: f64,
        pressure: f64,
        light: f64,
        battery: f64,
        signal_strength: i32,
        is_valid: bool,
    ) -> Self {
        Self {
            sensor_id,
            timestamp,
            temperature,
            humidity,
            pressure,
            light,
            battery,
            signal_strength,
            is_valid,
        }
    }
}

impl From<demo_api::SensorReading> for SensorReading {
    fn from(reading: demo_api::SensorReading) -> Self {
        Self {
            sensor_id: reading.sensor_id,
            timestamp: reading.timestamp,
            temperature: reading.temperature,
            humidity: reading.humidity,
            pressure: reading.pressure,
            light: reading.light,
            battery: reading.battery,
            signal_strength: reading.signal_strength,
            is_valid: reading.is_valid,
        }
    }
}

impl From<SensorReading> for demo_api::SensorReading {
    fn from(reading: SensorReading) -> Self {
        Self {
            sensor_id: reading.sensor_id,
            timestamp: reading.timestamp,
            temperature: reading.temperature,
            humidity: reading.humidity,
            pressure: reading.pressure,
            light: reading.light,
            battery: reading.battery,
            signal_strength: reading.signal_strength,
            is_valid: reading.is_valid,
        }
    }
}

#[wasm_bindgen]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
    pub timestamp: i64,
}

#[wasm_bindgen]
impl DataPoint {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64, timestamp: i64) -> Self {
        Self { x, y, timestamp }
    }
}

impl From<demo_api::DataPoint> for DataPoint {
    fn from(point: demo_api::DataPoint) -> Self {
        Self {
            x: point.x,
            y: point.y,
            timestamp: point.timestamp,
        }
    }
}

impl From<DataPoint> for demo_api::DataPoint {
    fn from(point: DataPoint) -> Self {
        Self {
            x: point.x,
            y: point.y,
            timestamp: point.timestamp,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl From<demo_api::Direction> for Direction {
    fn from(direction: demo_api::Direction) -> Self {
        match direction {
            demo_api::Direction::North => Self::North,
            demo_api::Direction::South => Self::South,
            demo_api::Direction::East => Self::East,
            demo_api::Direction::West => Self::West,
        }
    }
}

impl From<Direction> for demo_api::Direction {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::North => Self::North,
            Direction::South => Self::South,
            Direction::East => Self::East,
            Direction::West => Self::West,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserProfilePayload {
    id: i64,
    name: String,
    email: String,
    bio: String,
    age: i32,
    score: f64,
    tags: Vec<String>,
    scores: Vec<i32>,
    is_active: bool,
}

impl From<demo_api::BenchmarkUserProfile> for UserProfilePayload {
    fn from(profile: demo_api::BenchmarkUserProfile) -> Self {
        Self {
            id: profile.id,
            name: profile.name,
            email: profile.email,
            bio: profile.bio,
            age: profile.age,
            score: profile.score,
            tags: profile.tags,
            scores: profile.scores,
            is_active: profile.is_active,
        }
    }
}

impl From<UserProfilePayload> for demo_api::BenchmarkUserProfile {
    fn from(profile: UserProfilePayload) -> Self {
        Self {
            id: profile.id,
            name: profile.name,
            email: profile.email,
            bio: profile.bio,
            age: profile.age,
            score: profile.score,
            tags: profile.tags,
            scores: profile.scores,
            is_active: profile.is_active,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "tag")]
enum TaskStatusPayload {
    Pending,
    InProgress {
        progress: i32,
    },
    Completed {
        result: i32,
    },
    Failed {
        #[serde(rename = "errorCode")]
        error_code: i32,
        #[serde(rename = "retryCount")]
        retry_count: i32,
    },
}

impl From<TaskStatusPayload> for demo_api::TaskStatus {
    fn from(status: TaskStatusPayload) -> Self {
        match status {
            TaskStatusPayload::Pending => Self::Pending,
            TaskStatusPayload::InProgress { progress } => Self::InProgress { progress },
            TaskStatusPayload::Completed { result } => Self::Completed { result },
            TaskStatusPayload::Failed {
                error_code,
                retry_count,
            } => Self::Failed {
                error_code,
                retry_count,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DataPointPayload {
    x: f64,
    y: f64,
    timestamp: i64,
}

impl From<DataPointPayload> for demo_api::DataPoint {
    fn from(point: DataPointPayload) -> Self {
        Self {
            x: point.x,
            y: point.y,
            timestamp: point.timestamp,
        }
    }
}

fn deserialize_js<T>(value: JsValue, label: &str) -> T
where
    T: DeserializeOwned,
{
    serde_wasm_bindgen::from_value(value).unwrap_or_else(|error| panic!("invalid {label}: {error}"))
}

fn serialize_js<T>(value: &T, label: &str) -> JsValue
where
    T: Serialize,
{
    serde_wasm_bindgen::to_value(value)
        .unwrap_or_else(|error| panic!("failed to serialize {label}: {error}"))
}

#[wasm_bindgen]
extern "C" {
    pub type JsDataProvider;

    #[wasm_bindgen(method, js_name = getCount)]
    fn get_count(this: &JsDataProvider) -> u32;

    #[wasm_bindgen(method, js_name = getItem)]
    fn get_item(this: &JsDataProvider, index: u32) -> JsValue;
}

struct DataProviderBridge {
    provider: JsValue,
}

impl DataProviderBridge {
    fn new(provider: JsValue) -> Self {
        Self { provider }
    }
}

unsafe impl Send for DataProviderBridge {}
unsafe impl Sync for DataProviderBridge {}

impl demo_api::DataProvider for DataProviderBridge {
    fn get_count(&self) -> u32 {
        self.provider.unchecked_ref::<JsDataProvider>().get_count()
    }

    fn get_item(&self, index: u32) -> demo_api::DataPoint {
        let item = self
            .provider
            .unchecked_ref::<JsDataProvider>()
            .get_item(index);
        let point: DataPointPayload = deserialize_js(item, "data point");
        point.into()
    }
}

#[wasm_bindgen]
pub fn noop() {
    demo_api::noop()
}

#[wasm_bindgen]
pub fn echo_i32(value: i32) -> i32 {
    demo_api::echo_i32(value)
}

#[wasm_bindgen]
pub fn echo_bool(value: bool) -> bool {
    demo_api::echo_bool(value)
}

#[wasm_bindgen]
pub fn echo_f64(value: f64) -> f64 {
    demo_api::echo_f64(value)
}

#[wasm_bindgen]
pub fn negate_bool(value: bool) -> bool {
    demo_api::negate_bool(value)
}

#[wasm_bindgen]
pub fn echo_string(value: &str) -> String {
    demo_api::echo_string(value.to_owned())
}

#[wasm_bindgen]
pub fn echo_bytes(data: Vec<u8>) -> Vec<u8> {
    demo_api::echo_bytes(data)
}

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    demo_api::add(a, b)
}

#[wasm_bindgen]
pub fn add_f64(a: f64, b: f64) -> f64 {
    demo_api::add_f64(a, b)
}

#[wasm_bindgen]
pub fn multiply(a: f64, b: f64) -> f64 {
    demo_api::multiply(a, b)
}

#[wasm_bindgen]
pub fn generate_string(size: i32) -> String {
    demo_api::generate_string(size)
}

#[wasm_bindgen]
pub fn echo_vec_i32(values: Vec<i32>) -> Vec<i32> {
    demo_api::echo_vec_i32(values)
}

#[wasm_bindgen]
pub fn generate_locations(count: i32) -> Vec<Location> {
    demo_api::generate_locations(count)
        .into_iter()
        .map(Location::from)
        .collect()
}

#[wasm_bindgen]
pub fn find_locations(count: i32) -> Vec<Location> {
    demo_api::find_locations(count)
        .unwrap_or_default()
        .into_iter()
        .map(Location::from)
        .collect()
}

#[wasm_bindgen]
pub fn sum_location_ratings(locations: Vec<Location>) -> f64 {
    demo_api::sum_ratings(
        locations
            .into_iter()
            .map(demo_api::Location::from)
            .collect(),
    )
}

#[wasm_bindgen]
pub fn sum_ratings(locations: Vec<Location>) -> f64 {
    sum_location_ratings(locations)
}

#[wasm_bindgen]
pub fn process_locations(locations: Vec<Location>) -> i32 {
    demo_api::process_locations(
        locations
            .into_iter()
            .map(demo_api::Location::from)
            .collect(),
    )
}

#[wasm_bindgen]
pub fn generate_trades(count: i32) -> Vec<Trade> {
    demo_api::generate_trades(count)
        .into_iter()
        .map(Trade::from)
        .collect()
}

#[wasm_bindgen]
pub fn sum_trade_volumes(trades: Vec<Trade>) -> i64 {
    demo_api::sum_trade_volumes(trades.into_iter().map(demo_api::Trade::from).collect())
}

#[wasm_bindgen]
pub fn generate_particles(count: i32) -> Vec<Particle> {
    demo_api::generate_particles(count)
        .into_iter()
        .map(Particle::from)
        .collect()
}

#[wasm_bindgen]
pub fn sum_particle_masses(particles: Vec<Particle>) -> f64 {
    demo_api::sum_particle_masses(
        particles
            .into_iter()
            .map(demo_api::Particle::from)
            .collect(),
    )
}

#[wasm_bindgen]
pub fn generate_sensor_readings(count: i32) -> Vec<SensorReading> {
    demo_api::generate_sensor_readings(count)
        .into_iter()
        .map(SensorReading::from)
        .collect()
}

#[wasm_bindgen]
pub fn avg_sensor_temperature(readings: Vec<SensorReading>) -> f64 {
    demo_api::avg_sensor_temperature(
        readings
            .into_iter()
            .map(demo_api::SensorReading::from)
            .collect(),
    )
}

#[wasm_bindgen]
pub fn generate_bytes(size: i32) -> Vec<u8> {
    demo_api::generate_bytes(size)
}

#[wasm_bindgen]
pub fn generate_i32_vec(count: i32) -> Vec<i32> {
    demo_api::generate_i32_vec(count)
}

#[wasm_bindgen]
pub fn sum_i32_vec(values: Vec<i32>) -> i64 {
    demo_api::sum_i32_vec(values)
}

#[wasm_bindgen]
pub fn generate_f64_vec(count: i32) -> Vec<f64> {
    demo_api::generate_f64_vec(count)
}

#[wasm_bindgen]
pub fn sum_f64_vec(values: Vec<f64>) -> f64 {
    demo_api::sum_f64_vec(values)
}

#[wasm_bindgen]
pub fn inc_u64(mut values: Vec<u64>) -> Vec<u64> {
    demo_api::inc_u64(&mut values);
    values
}

#[wasm_bindgen]
pub fn inc_u64_value(value: u64) -> u64 {
    demo_api::inc_u64_value(value)
}

#[wasm_bindgen]
pub fn opposite_direction(direction: Direction) -> Direction {
    demo_api::opposite_direction(direction.into()).into()
}

#[wasm_bindgen]
pub fn direction_to_degrees(direction: Direction) -> i32 {
    demo_api::direction_to_degrees(direction.into())
}

#[wasm_bindgen]
pub fn generate_directions(count: i32) -> Vec<Direction> {
    demo_api::generate_directions(count)
        .into_iter()
        .map(Direction::from)
        .collect()
}

#[wasm_bindgen]
pub fn count_north(directions: Vec<Direction>) -> i32 {
    demo_api::count_north(
        directions
            .into_iter()
            .map(demo_api::Direction::from)
            .collect(),
    )
}

#[wasm_bindgen]
pub fn echo_direction(direction: Direction) -> Direction {
    demo_api::echo_direction(direction.into()).into()
}

#[wasm_bindgen]
pub fn find_direction(id: i32) -> Option<Direction> {
    demo_api::find_direction(id).map(Direction::from)
}

#[wasm_bindgen]
pub struct Counter {
    inner: demo_api::Counter,
}

#[wasm_bindgen]
impl Counter {
    #[wasm_bindgen(constructor)]
    pub fn new(initial: i32) -> Self {
        Self {
            inner: demo_api::Counter::new(initial),
        }
    }

    pub fn increment(&self) {
        self.inner.increment();
    }

    pub fn get(&self) -> i32 {
        self.inner.get()
    }

    pub fn add(&self, amount: i32) {
        self.inner.add(amount);
    }

    pub fn reset(&self) {
        self.inner.reset();
    }
}

#[wasm_bindgen]
pub struct CounterSingleThreaded {
    inner: demo_api::CounterSingleThreaded,
}

#[wasm_bindgen]
impl CounterSingleThreaded {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: demo_api::CounterSingleThreaded::new(),
        }
    }

    pub fn increment(&mut self) {
        self.inner.increment();
    }

    pub fn get(&self) -> i32 {
        self.inner.get()
    }

    pub fn set(&mut self, value: i32) {
        self.inner.set(value);
    }
}

#[wasm_bindgen]
pub struct DataStore {
    inner: demo_api::DataStore,
}

#[wasm_bindgen]
impl DataStore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: demo_api::DataStore::new(),
        }
    }

    pub fn add(&self, point: DataPoint) {
        self.inner.add(point.into());
    }

    pub fn add_parts(&self, x: f64, y: f64, timestamp: i64) {
        self.inner.add_parts(x, y, timestamp);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn sum(&self) -> f64 {
        self.inner.sum()
    }
}

#[wasm_bindgen]
pub struct Accumulator {
    inner: demo_api::Accumulator,
}

#[wasm_bindgen]
impl Accumulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: demo_api::Accumulator::new(),
        }
    }

    pub fn add(&self, amount: i64) {
        self.inner.add(amount);
    }

    pub fn get(&self) -> i64 {
        self.inner.get()
    }

    pub fn reset(&self) {
        self.inner.reset();
    }
}

#[wasm_bindgen]
pub struct AccumulatorSingleThreaded {
    inner: demo_api::AccumulatorSingleThreaded,
}

#[wasm_bindgen]
impl AccumulatorSingleThreaded {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: demo_api::AccumulatorSingleThreaded::new(),
        }
    }

    pub fn add(&mut self, amount: i64) {
        self.inner.add(amount);
    }

    pub fn get(&self) -> i64 {
        self.inner.get()
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }
}

#[wasm_bindgen]
pub struct DataConsumer {
    inner: demo_api::DataConsumer,
}

#[wasm_bindgen]
impl DataConsumer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: demo_api::DataConsumer::new(),
        }
    }

    pub fn set_provider(&self, provider: JsValue) {
        self.inner
            .set_provider(Box::new(DataProviderBridge::new(provider)));
    }

    pub fn compute_sum(&self) -> u64 {
        self.inner.compute_sum()
    }
}

#[wasm_bindgen]
pub fn generate_user_profiles(count: i32) -> JsValue {
    let profiles: Vec<UserProfilePayload> = demo_api::generate_user_profiles(count)
        .into_iter()
        .map(UserProfilePayload::from)
        .collect();
    serialize_js(&profiles, "user profiles")
}

#[wasm_bindgen]
pub fn sum_user_scores(users: JsValue) -> f64 {
    let profiles: Vec<UserProfilePayload> = deserialize_js(users, "user profiles");
    demo_api::sum_user_scores(
        profiles
            .into_iter()
            .map(demo_api::BenchmarkUserProfile::from)
            .collect(),
    )
}

#[wasm_bindgen]
pub fn count_active_users(users: JsValue) -> i32 {
    let profiles: Vec<UserProfilePayload> = deserialize_js(users, "user profiles");
    demo_api::count_active_users(
        profiles
            .into_iter()
            .map(demo_api::BenchmarkUserProfile::from)
            .collect(),
    )
}

#[wasm_bindgen]
pub fn get_status_progress(status: JsValue) -> i32 {
    let status: TaskStatusPayload = deserialize_js(status, "task status");
    demo_api::get_status_progress(status.into())
}

#[wasm_bindgen]
pub fn is_status_complete(status: JsValue) -> bool {
    let status: TaskStatusPayload = deserialize_js(status, "task status");
    demo_api::is_status_complete(status.into())
}

#[wasm_bindgen]
pub async fn async_add(a: i32, b: i32) -> i32 {
    demo_api::async_add(a, b).await
}

#[wasm_bindgen]
pub fn find_name(id: i32) -> Option<String> {
    demo_api::find_name(id)
}

#[wasm_bindgen]
pub fn find_names(count: i32) -> Vec<String> {
    demo_api::find_names(count).unwrap_or_default()
}

#[wasm_bindgen]
pub fn find_numbers(count: i32) -> Vec<i32> {
    demo_api::find_numbers(count).unwrap_or_default()
}

#[wasm_bindgen]
pub fn find_even(value: i32) -> Option<i32> {
    demo_api::find_even(value)
}

#[wasm_bindgen]
pub fn find_positive_f64(value: f64) -> Option<f64> {
    demo_api::find_positive_f64(value)
}
