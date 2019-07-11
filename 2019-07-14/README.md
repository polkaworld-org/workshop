# Substrate 智能合约初窥

<!-- TOC GFM -->

* [环境准备](#环境准备)
    * [编译 Substrate](#编译-substrate)
    * [WASM 构建环境](#wasm-构建环境)
    * [安装 ink!](#安装-ink)
        * [直接安装二进制](#直接安装二进制)
        * [编译安装](#编译安装)
    * [其他](#其他)

<!-- /TOC -->

## 环境准备

### 编译 Substrate

参见 [Hacking on Substrate](https://github.com/paritytech/substrate#building)

### WASM 构建环境

Substrate合约需要编译为 WASM 形式进行部署，需要安装 wasm32-unknown-unknown 的 target, 将Rust代码编译为WASM. 由于 ink! 目前将 rust-toolchain 指定为 2019-05-21, 我们还必须要安装这个日期的 wasm32-unknown-unknown.

```bash
rustup install nightly-2019-05-21
rustup target add wasm32-unknown-unknown --toolchain nightly-2019-05-21
```

[wabt](https://github.com/WebAssembly/wabt) 是一个 WASM 的工具包, 可以将 wasm 文件与其他文件格式进行互相转换，比如将wasm的文本格式转换成二进制格式.

```bash
brew install wabt
```

[wasm-utils](https://github.com/paritytech/wasm-utils) 是一个 parity 开发的一个 WASM 工具包，`wasm-prune` 可以对编译好的 wasm 文件进行修剪，只保留合约 `call` 会用到的元素, 减少链上存储。

```bash
cargo install pwasm-utils-cli --bin wasm-prune
```

### 安装 ink!

#### 直接安装二进制

```bash
cargo install --force --git https://github.com/paritytech/ink cargo-contract
```

通过 ink! 创建一个新的模板合约项目:

```bash
cargo contract new flipper 
```

#### 编译安装

```bash
git clone https://github.com/paritytech/ink
cd ink
cargo build
```

通过 ink! 创建一个新的模板合约项目:

```bash
./ink/target/debug/cargo-contract contract new flipper
```

### 其他

ink!使用了一个非常重要的宏 `contract!`, 可以将这个宏展开进一步分析。[cargo-expand](https://github.com/dtolnay/cargo-expand) 是一个用于展开 Rust 宏的工具，可以通过 `cargo install cargo-expand` 进行安装, `cargo-expand` 主要封装了这个命令 `cargo rustc --profile=check -- -Zunstable-options --pretty=expanded`, 如果不想安装 `cargo-expand`, 直接使用这个命令展开也可以。

另外，安装 rustfmt 用于格式化宏展开后的代码方便阅读:

```bash
rustup component add rustfmt --toolchain nightly-2019-05-21-x86_64-apple-darwin
```

References:

- https://substrate.dev/substrate-contracts-workshop/#/0/setup
