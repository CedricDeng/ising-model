# 可验证伊辛模型模拟平台 (Verifiable Ising Model)

本项目是一个基于 **SP1 zkVM** 的科研实验示例，旨在解决科学模拟中的数据可信度问题。通过零知识证明（ZKP）技术，我们实现了实验过程的“全链条诚信”：

1.  **方法不可篡改 (Case 1)**：通过 Verification Key (VK) 锁定物理公式，防止事后修改物理常数。
2.  **结果不可篡改 (Case 2)**：数学证明输出的能量与磁化强度确实由声明的代码生成，无法手动改数。

---

## 📋 1. 环境准备

本实验建议在高性能 Linux 服务器（如 Ubuntu 22.04）上运行。

### 1.1 基础环境安装
```bash
# 1. 安装 Rust 编译器
curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh
source $HOME/.cargo/env
```

# 2. 安装 SP1 工具链
```
curl -L [https://sp1up.succinct.xyz](https://sp1up.succinct.xyz) | bash
source $HOME/.bashrc
sp1up
```

### 1.2 安装Docker
生成Plonk类型的证明需要Docker环境
```
sudo apt update && sudo apt install docker.io -y
sudo systemctl start docker
sudo usermod -aG docker $USER
```

## 2. 项目构建与编译
项目结构分为三个部分:
- ising-lib: 基础物理公式与数据结构定义
- ising-program: 在zkvm中运行的物理内核
- script: 负责驱动模拟并产出证明文件

在项目根目录运行:
```
cd ising-program
cargo prove build
```

这段代码会产出Verification Key（VK），需要记录在论文中

## 3. 运行实验
以下代码运行计算，可以看到输出，用于调试
```
cd ../script
cargo run --release --bin ising-sim -- --execute
```

以下代码生成可供验证的证明包（最耗时部分），产出为json文件
```
cargo run --release --bin ising-sim -- --prove
```

