# 核电厂热力计算程序

本项目是一个基于 Rust 和 Slint 构建的核电厂热力循环计算桌面应用程序。它允许用户输入详细的运行参数，执行热力计算，查看计算结果，并能生成相应的计算过程代码。项目源于[euaurora/curriculum-design2](https://gitee.com/euaurora/curriculum-design2)的启发。

## 主要功能

*   **参数输入**: 提供图形化界面，方便用户输入或修改核电厂热力系统的各项参数。
*   **热力计算**: 执行复杂的热力循环计算，包括蒸汽参数、各级抽汽、给水加热、再热等。
*   **结果展示**: 清晰展示计算的中间过程数据和最终结果，如功率、效率、各点焓熵值等。
*   **参数管理**:
    *   支持将输入的参数保存到 JSON 文件 ([parameters.json](parameters.json))。
    *   支持从 JSON 文件加载参数。
*   **结果保存**: 支持将计算结果保存到 JSON 文件 ([results.json](results.json))。
*   **代码生成**:
    *   能够生成 Rust 版本的计算过程代码 (参见 [`Calculator::generate_calc_code_rs`](calc/src/lib.rs))。
    *   能够生成 Python 版本的计算过程代码 (参见 [`Calculator::generate_calc_code_py`](calc/src/lib.rs))。
*   **配置保存**: 应用程序配置（如输出目录）可保存到 [config.json](config.json)。

## 项目结构

```
.
├── calc/                   #核心计算逻辑模块
│   ├── src/
│   │   ├── lib.rs          # 计算逻辑实现
│   │   └── parameters.rs   # 参数及结果数据结构定义
│   └── Cargo.toml
├── src/                    #主程序源码
│   ├── main.rs             # 程序入口及UI逻辑
│   └── config.rs           # 配置相关结构
├── ui/                     # Slint UI 定义
│   ├── app.slint           # 主窗口UI
│   ├── components/         # UI组件
│   ├── assets/             # UI资源文件
│   └── models/             # UI数据模型
├── Cargo.toml              # Rust项目配置
├── build.rs                # Slint编译脚本
├── parameters.json         # 示例输入参数文件
│   └── models/             # UI数据模型
├── Cargo.toml              # Rust项目配置
├── build.rs                # Slint编译脚本
├── parameters.json         # 示例输入参数文件
├── results.json            # 示例结果文件
└── README.md               # 本文档
```

## 构建与运行

### 环境要求

*   Rust 工具链
*   Slint UI 库的相关依赖

### 构建

在项目根目录下执行：

```sh
cargo build --release
```

### 运行

```sh
cargo run
```

或者直接运行 `target/release/` 目录下的可执行文件。

## 使用说明

1.  启动应用程序。
2.  在 "输入参数" 标签页中手动输入参数，或通过 "文件" -> "加载参数..." 菜单从 `parameters.json` 文件加载，或通过 "计算" -> "加载默认值..." 菜单加载默认值再进行修改。
3.  通过 "文件" -> "选择输出目录..." 菜单设置结果和代码的保存路径。
4.  点击 "计算" -> "开始计算" 按钮执行热力计算。
5.  计算完成后，可在 "计算结果" 标签页查看详细结果。
6.  生成的 Rust 计算过程代码会显示在 "计算代码(Rust)" 标签页。
7.  生成的 Python 计算过程代码会显示在 "计算代码(Python)" 标签页。
8.  可使用 "文件" 菜单保存当前参数或计算结果，或使用 "计算" 菜单保存生成的计算代码。

## 依赖库

*   `slint`：用于构建图形用户界面。
*   `serde` (`serde_json`)：用于参数和结果的 JSON 序列化与反序列化。
*   `rfd` ：用于文件对话框。

## 计划开发
- [ ] 热力系统原理图自动生成

## 贡献

欢迎提交问题报告或功能请求。

## 许可证

[LICENSE](LICENSE)