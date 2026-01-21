#![no_main]
sp1_zkvm::entrypoint!(main);

use ising_lib::{run_simulation, IsingInput, PublicValuesStruct};
use alloy_sol_types::SolType;

pub fn main() {
    // 1. 从 Host 读取实验输入 (温度和种子)
    let input = sp1_zkvm::io::read::<IsingInput>();

    // 2. 运行你在 lib.rs 中写的物理模拟
    let (energy, mag) = run_simulation(&input);

    // 3. 按照 ABI 规范编码输出数据，防止 Case 3 (结果篡改)
    let public_values = PublicValuesStruct {
        seed: input.seed,
        temperature_fixed: (input.temperature * 100.0) as u32,
        avg_energy: energy,
        avg_mag: mag,
    };
    let bytes = PublicValuesStruct::abi_encode(&public_values);

    // 4. 将结果提交至证明
    sp1_zkvm::io::commit_slice(&bytes);
}