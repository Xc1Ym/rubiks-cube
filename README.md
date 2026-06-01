# 🎲 Rust 魔方

一个用 Rust 编写的交互式魔方模拟器，支持 3D 立体视图和 2D 平面展开图。

## ✨ 功能特性

- **3D 立体视图**：可拖拽旋转视角，点击面进行旋转操作
- **2D 平面展开图**：十字架布局，点击任意面执行旋转
- **平滑动画**：单步 90°/180° 旋转插值动画
- **公式系统**：支持标准魔方记法（如 `R U R' U'`），自动播放
- **打乱功能**：随机生成 1~100 步合法打乱序列
- **撤销/重做**：完整的历史栈管理

## 🚀 快速开始

### 从源码构建

```bash
git clone https://github.com/ligong/rubiks-cube.git
cd rubiks-cube
cargo run --release
```

### 预编译二进制文件

从 [Releases](https://github.com/ligong/rubiks-cube/releases) 页面下载对应平台的可执行文件。

| 平台 | 架构 | 下载 |
|------|------|------|
| macOS | ARM64 (Apple Silicon) | `.tar.gz` |
| Windows | x64 | `.zip` |
| Windows | ARM64 | `.zip` |

## 🎮 操作说明

| 操作 | 说明 |
|------|------|
| **3D 视图拖拽** | 旋转观察视角 |
| **3D 视图点击** | 顺时针旋转该面 |
| **Shift + 3D 点击** | 逆时针旋转该面 |
| **2D 展开图点击** | 顺时针旋转该面 |
| **公式输入** | 输入如 `R U R' U'`，点击执行 |

## 🛠 技术栈

- **GUI 框架**：[egui](https://github.com/emilk/egui) + [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
- **3D 渲染**：纯 CPU 顶点投影，不依赖外部 3D 引擎
- **数学基础**：群论置换群 + 右手定则旋转

## 📁 项目结构

```
rubiks-cube/
├── src/
│   ├── main.rs              # 程序入口
│   ├── app.rs               # GUI 应用主逻辑
│   ├── cube/
│   │   ├── mod.rs           # 魔方核心状态机
│   │   └── moves.rs         # 标准记法解析
│   └── renderer/
│       ├── mod.rs           # 渲染公共工具
│       ├── view_2d.rs       # 2D 展开图渲染
│       └── view_3d.rs       # 3D 立体视图渲染
├── .github/workflows/
│   └── release.yml          # 自动发布 CI
└── Cargo.toml
```

## 📄 许可证

MIT License
