use clap::Parser;
// 1. 必须导入 HashableKey 才能使用 .bytes32()
use sp1_sdk::{ProverClient, SP1Stdin, HashableKey}; 
use ising_lib::{IsingInput, PublicValuesStruct};
use alloy_sol_types::SolType;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    prove: bool,

    #[arg(long)]
    execute: bool,

    #[arg(long, default_value = "42")]
    seed: u64,
}

fn main() {
    sp1_sdk::utils::setup_logger();
    let args = Args::parse();

    // 路径：bin -> src -> script -> ising-sim 根目录
    const ISING_ELF: &[u8] = include_bytes!("../../../target/elf-compilation/riscv32im-succinct-zkvm-elf/release/ising-program");
    
    // 2. 使用推荐的 from_env() 替代 new()
    let client = ProverClient::from_env();
    let (pk, vk) = client.setup(ISING_ELF);

    let temperatures = vec![1.5, 2.0, 2.26, 3.0];

    println!("====================================================");
    // 现在 .bytes32() 可以正常使用了
    println!("实验方法指纹 (VK): {:?}", vk.bytes32());
    println!("使用种子: {}", args.seed);
    println!("====================================================");

    for t in temperatures {
        let mut stdin = SP1Stdin::new();
        stdin.write(&IsingInput { seed: args.seed, temperature: t });

        if args.prove {
            println!("\n[证明模式] 正在为 T = {} 生成零知识证明...", t);
            // 3. 必须传引用 &stdin
            let proof = client.prove(&pk, &stdin).plonk().run().expect("Proving failed");
            
            let bytes = proof.public_values.as_slice();
            // 4. abi_decode 只接受一个参数，去掉 true
            let output = PublicValuesStruct::abi_decode(bytes).unwrap();
            
            // 5. 修正字段名：使用 avg_energy 和 avg_mag
            println!("✅ 证明生成成功！物理结果：平均能量={}, 平均磁化={}", output.avg_energy, output.avg_mag);

            let filename = format!("proof_T_{}.json", t);
            proof.save(&filename).unwrap();
            println!("💾 证据包已存至: {}", filename);

        } else if args.execute {
            println!("\n[执行模式] 正在快速计算 T = {} 的结果...", t);
            // 6. 必须传引用 &stdin
            let (public_values_bytes, report) = client.execute(ISING_ELF, &stdin).run().expect("Execution failed");
            
            let output = PublicValuesStruct::abi_decode(public_values_bytes.as_slice()).unwrap();
            // 7. 修正字段名：使用 avg_energy 和 avg_mag
            println!("📊 计算完成！平均能量={}, 平均磁化={}", output.avg_energy, output.avg_mag);
            println!("⚡ 消耗指令数: {}", report.total_instruction_count());
        }
    }
}