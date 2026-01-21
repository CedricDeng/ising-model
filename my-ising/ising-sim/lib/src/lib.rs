use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};

// ============================================================
// 1. 定义数据结构
// ============================================================

#[derive(Serialize, Deserialize, Clone)]
pub struct IsingInput {
    pub seed: u64,
    pub temperature: f64,
}

sol! {
    /// 这里的定义要与你在验证平台上展示的数据完全对齐
    struct PublicValuesStruct {
        uint64 seed;
        uint32 temperature_fixed; // 温度放大100倍存为整数
        int32 avg_energy;
        int32 avg_mag;
    }
}

// ============================================================
// 2. 模拟核心组件 (保持原算法逻辑)
// ============================================================

const N: usize = 8; 
const EQ_STEPS: usize = 20; 
const MC_STEPS: usize = 20;

type SpinConfig = [[i8; N]; N];

struct PRNG {
    state: u64,
}

impl PRNG {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        ((self.state.wrapping_mul(0x2545F4914F6CDD1D)) >> 32) as u32
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u32() as f64) / (u32::MAX as f64)
    }

    fn range(&mut self, n: usize) -> usize {
        (self.next_u32() as usize) % n
    }

    fn next_bool(&mut self) -> bool {
        self.next_u32() & 1 == 0
    }
}

fn init_spin(prng: &mut PRNG) -> SpinConfig {
    let mut config = [[0i8; N]; N];
    for i in 0..N {
        for j in 0..N {
            config[i][j] = if prng.next_bool() { 1 } else { -1 };
        }
    }
    config
}

fn mc_step(config: &mut SpinConfig, beta: f64, prng: &mut PRNG) {
    let i = prng.range(N);
    let j = prng.range(N);
    let s = config[i][j];
    let neighbor_sum =
        config[(i + 1) % N][j] as i32 +
        config[(i + N - 1) % N][j] as i32 +
        config[i][(j + 1) % N] as i32 +
        config[i][(j + N - 1) % N] as i32;

    let delta_e = 2.0 * (s as f64) * (neighbor_sum as f64);

    if delta_e < 0.0 || prng.next_f64() < (-beta * delta_e).exp() {
        config[i][j] = -s;
    }
}

fn energy_calc(config: &SpinConfig) -> i32 {
    let mut e = 0;
    for i in 0..N {
        for j in 0..N {
            let s = config[i][j] as i32;
            let right = config[(i + 1) % N][j] as i32;
            let down = config[i][(j + 1) % N] as i32;
            e -= s * (right + down);
        }
    }
    e
}

fn magnetization_calc(config: &SpinConfig) -> i32 {
    config.iter().flatten().map(|&s| s as i32).sum()
}

// ============================================================
// 3. 供 Guest 和 Host 调用的主模拟接口
// ============================================================

/// 执行物理模拟并返回平均能量和平均磁化强度
pub fn run_simulation(input: &IsingInput) -> (i32, i32) {
    let beta: f64 = 1.0 / input.temperature;
    let mut prng = PRNG::new(input.seed);
    let mut config = init_spin(&mut prng);

    // 平衡阶段
    for _ in 1..=EQ_STEPS {
        mc_step(&mut config, beta, &mut prng);
    }

    // 测量阶段
    let mut energy_sum = 0;
    let mut mag_sum = 0;

    for _ in 1..=MC_STEPS {
        mc_step(&mut config, beta, &mut prng);
        energy_sum += energy_calc(&config);
        mag_sum += magnetization_calc(&config).abs(); // 取绝对值平均
    }

    // 返回累计和（在 Guest 内部处理成平均值或保持整数以配合 ABI）
    let avg_energy = energy_sum / (MC_STEPS as i32);
    let avg_mag = mag_sum / (MC_STEPS as i32);

    (avg_energy, avg_mag)
}