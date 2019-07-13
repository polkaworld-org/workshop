# Substrate 智能合约初窥

<!-- TOC GFM -->

* [环境准备](#环境准备)
    * [编译 Substrate](#编译-substrate)
    * [WASM 构建环境](#wasm-构建环境)
    * [安装 ink!](#安装-ink)
        * [直接安装二进制](#直接安装二进制)
        * [编译安装](#编译安装)
    * [其他](#其他)
* [使用ink!编写合约](#使用ink编写合约)
    * [ink!模块](#ink模块)
    * [编译合约](#编译合约)
    * [合约测试](#合约测试)
* [部署合约](#部署合约)
    * [启动本地 dev 测试链](#启动本地-dev-测试链)
    * [连接 dev 测试链](#连接-dev-测试链)
    * [正式部署合约](#正式部署合约)
        * [1. 上传WASM合约代码: upload](#1-上传wasm合约代码-upload)
        * [2. 实例化合约账户: deploy](#2-实例化合约账户-deploy)
        * [3. 合约调用: execute](#3-合约调用-execute)
* [Permissioned Flipper](#permissioned-flipper)
* [合约进阶小练习](#合约进阶小练习)
* [系统合约与用户合约](#系统合约与用户合约)
    * [Runtime模块交互](#runtime模块交互)
    * [可升级性](#可升级性)

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

## 使用ink!编写合约

### ink!模块

```bash
ink
├── alloc
├── cli
├── core
├── examples
├── lang
├── model
└── utils
```

- cli: 用于快速创建一个合约工程。
- core: 写合约的一些核心工具，包括存储分配，封装SRML的类型定义等。
- examples: cli, core, lang 三个层面的示例代码。
- lang: 基于 core 和 model 提供了 contract! 宏，使编写合约更加友好。
- model: 对合约概念进行了初步封装，
- utils: 一些实用工具。

core, model, lang 不同层面的抽象，都可以用来编写合约，但是 lang 模块最友好，抽象程度最高，它提供了一个非常重要的 `contract!`, 这个宏展开后实际上是对 core 和 model 的一些封装。

新建合约项目：

```bash
cargo contract new flipper
```

通过链下模拟的环境测试合约功能是否正常：

```bash
cargo test --features test-env
```

宏展开后的合约代码：

```bash
cargo rustc -- -Z unstable-options --pretty=expanded > flipper.rs
rustfmt flipper.rs
```

### 编译合约

执行 `build.sh` 将合约编译到WASM, 这也意味着必须要在 no_std 的环境下编译，std 的库都用不了。目前能够用的库就是 `core` 和 `alloc` 这两个, 如果要在合约中引入第三方库，那么必须保证第三方库支持 no_std, 能够在 no_std 下编译。

```bash
bash build.sh
```

编译输出都在 target 目录下:

```
target
├── Flipper.json
├── debug
├── flipper-fixed.wat
├── flipper-pruned.wasm
├── flipper.wasm
├── flipper.wat
├── release
└── wasm32-unknown-unknown
```

编译脚本解析：

```bash
#!/bin/bash
set -e

PROJNAME=flipper

# cargo clean
# rm Cargo.lock

#### 关闭增量编译, 并将合约编译为WASM
CARGO_INCREMENTAL=0 &&
cargo build --release --features generate-api-description --target=wasm32-unknown-unknown --verbose

#### 将编译好的wasm文件输出为wat格式
wasm2wat -o target/$PROJNAME.wat target/wasm32-unknown-unknown/release/$PROJNAME.wasm

#### 添加 wasm 内存声明的最大值，如果不指定最大值的话，默认是 4GB.
cat target/$PROJNAME.wat | sed "s/(import \"env\" \"memory\" (memory (;0;) 2))/(import \"env\" \"memory\" (memory (;0;) 2 16))/" > target/$PROJNAME-fixed.wat

#### 将添加最大值声明的wat充值导出为wasm格式
wat2wasm -o target/$PROJNAME.wasm target/$PROJNAME-fixed.wat

#### 对wasm文件进行修剪，修剪后的wasm文件就是我们要最终上传到链上的文件
wasm-prune --exports call,deploy target/$PROJNAME.wasm target/$PROJNAME-pruned.wasm
```

`Flipper.json` 就是合约的ABI，合约暴露出来所有 public 函数都会在里面.

`selector` 是函数名的hash值，用来路由函数。

### 合约测试

## 部署合约

### 启动本地 dev 测试链

所谓部署合约，是要将合约部署到区块链上，那么首先要一条链, 我们起一条 substrate 的 dev 测试链。首先下载最新的 [substrate](https://github.com/paritytech/substrate) 代码并切换到一个指定的 617a90e89, 大概是一周前的版本。因为 master 分支是 substrate 开发迭代的分支，很容易出现 breaking change, 导致各种问题。我这里随机选择了一个"能用"的版本，如果直接最新版本的substrate, 可能会出现一些问题。另外，最好编译release版本, 因为debug版本性能很差，有时候可能第一个块都出不来。

```bash
# 首先下载最新的susbtrate代码，然后切到一个指定的 commit. 
git clone https://github.com/paritytech/substrate
git checkout 617a90e89
cargo build --release
```

由于 substrate 仍然在快速迭代，包括前端等都有可能会有bug存在。比如目前 Polkadot UI 对 substrate 2.0 还有些bug, 在实例化合约时，endowment输入框有时候无法输入正确的数值。直接修改substrate代码绕过这个这个问题:(, 将 `ExistentialDeposit` 设置为0.

```diff
diff --git a/node/runtime/src/lib.rs b/node/runtime/src/lib.rs
index afc70e871..4d644ca69 100644
--- a/node/runtime/src/lib.rs
+++ b/node/runtime/src/lib.rs
@@ -136,7 +136,7 @@ impl indices::Trait for Runtime {
 }

 parameter_types! {
-       pub const ExistentialDeposit: Balance = 1 * DOLLARS;
+       pub const ExistentialDeposit: Balance = 0;
        pub const TransferFee: Balance = 1 * CENTS;
        pub const CreationFee: Balance = 1 * CENTS;
        pub const TransactionBaseFee: Balance = 1 * CENTS;
```

编译好后启动 dev 模式测试链:

```bash
./target/release/substrate --dev
```

如果启动 `--dev` 模式出现问题，可以尝试先清除数据再重新启动。

```bash
# 使用 purge-chain 自动清除数据
./target/release/substrate purge-chain --dev
```

或者选择手动清除数据，macOS 数据默认存放路径:

```bash
rm -rf $HOME/Library/Application Support/substrate/chains/dev/db
```

### 连接 dev 测试链

打开 https://polkadot.js.org/apps/#/settings 设置连接到本地的测试链。

### 正式部署合约

合约部署分为 3 个步骤 upload -> deploy -> execute:

1. 上传WASM合约代码。
2. 通过链上存储的WASM代码实例化一个合约账户。
3. 合约调用。

#### 1. 上传WASM合约代码: upload

https://polkadot.js.org/apps/#/contracts/code

首先通过调用 contracts 模块的 `put_code`，上传WASM合约代码，后续通过 code_hash 使用。如果多个合约的逻辑安全一样，只有初始化参数或者说合约的"构造函数"不一样，那么可以共用一份逻辑代码，减少链上存储。

#### 2. 实例化合约账户: deploy

#### 3. 合约调用: execute

当一个合约被调用时，对应的合约代码会通过 code_hash 拿到并执行。它可以改变合约存储，创建新合约，也可以调用其他合约。

当合约账户被清除时，相关的代码和存储也会被存储。

## Permissioned Flipper

通过 ink! 创建的示例项目 flipper, 任何人都可以玩，参与游戏的权限是 permissionless, 那么如果我们想要改成 permissioned ，即这个合约可以有一个拥有者owner和几个管理员admin，只有这些管理员和拥有者才有权限 flip。

练习代码在 [permissionedflipper](./permissionedflipper).

## 合约进阶小练习

加入”付费“元素. 除了 owner 和 admin, 如果一个用户拥有足够参与这个游戏的 token, 就可以执行 flip 操作。手续费 token 可以在 Flipper 合约中内置，参考 erc20 示例进行实现。除了自定义token, 等
https://github.com/paritytech/ink/pull/124 合并以后，还可以直接使用系统代币 coin, 直接扣系统代币参与游戏 flip.

## 系统合约与用户合约

- SRML
- Contract

```
contract => contract storage
runtime  => Blockchain storage
```

### Runtime模块交互

- 调用Runtime函数

    - https://github.com/paritytech/ink/pull/124
    - https://github.com/paritytech/ink/pull/135

- 读取Runtime存储

### 可升级性

- Contract: 不可升级
- SRML: 可升级

系统合约升级 -> `set_code`

References:

- https://substrate.dev/substrate-contracts-workshop/

