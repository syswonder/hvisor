# Commit `40d2868`：从 Cargo `feature` 到 Kconfig 的完整说明

本文档基于 commit `40d2868cf97a96ca9c98d61430f90ec52375e22d`（message: `kconfig`），说明该提交如何将平台能力开关从 **Cargo `[features]` + 每板 `cargo/features` 文件** 迁移到 **Linux 风格的 Kconfig（`kconfig/Kconfig` + 每板 `kconfig/defconfig`）+ 根目录 `.config`**，以及各新增/修改文件的含义与必要性。

与日常用法更短的操作说明见同目录下的 [`kconfig.md`](./kconfig.md)。

---

## 一、动机与总体架构变化

### 1.1 旧模型（迁移前）

- **`Cargo.toml` `[features]`**：声明大量空 feature（如 `pci`、`gicv3`），仅用于在编译期打开 `#[cfg(feature = "...")]`。
- **`platform/<arch>/<board>/cargo/features`**：每行一个 feature 名；`Makefile` 通过 `tools/read_features.sh` 读入，拼成 `cargo build --features "..."`。
- **局限**：无统一菜单、无 `depends on` / `select` 语义、互斥关系只能靠文档或运行时；与内核/固件生态的 Kconfig 习惯不一致；`rust-analyzer` 与真实 `--features` 易脱节。

### 1.2 新模型（迁移后）

| 层级 | 作用 |
|------|------|
| `kconfig/Kconfig` | 声明 `config` 符号、菜单、`default` / `depends on` / `select`（由 kconfiglib 解析）。 |
| `platform/.../kconfig/defconfig` | 每板默认打开的 `CONFIG_*=y` 列表（可版本管理）。 |
| 仓库根 `.config` | 生成物：前几行 **元数据**（`ARCH`/`BOARD`/`BID`/`HVISOR_SRC`/`LD_SCRIPT`/`TEMPLATE`）+ Kconfig 展开后的 `CONFIG_*` 行；**不入库**（`.gitignore`）。 |
| `kconfig/cfg_map.toml` | **`CONFIG_*` → Rust `cfg` 名** 的唯一映射表（与源码中 `#[cfg(...)]` 一致）。 |
| `build.rs` | 读 `.config` 与 `cfg_map.toml`，对启用的项打印 `cargo:rustc-cfg=...`，并输出 `cargo:rustc-check-cfg`；继续负责 `src/platform/__board.rs` 符号链接。 |
| `Makefile` | 在 `all`/`elf` 等目标前 **`ensure_config`**：若无 `.config` 则先 `defconfig`；**不再**向 cargo 传递 `--features`。 |

**必要性**：把「板级选了哪些驱动/子系统」从 Cargo 的 package 级 feature 机制中解耦出来，换成与内核类似的配置表面，便于依赖约束、交互式修改、CI 矩阵统一用 `make defconfig` 生成 `.config`。

---

## 二、Cargo 与构建入口改动

### 2.1 删除 `Cargo.toml` 中的 `[features]` 整块

原先约 56 行 `[features]`（`iommu`、`pci`、`gicv2`/`gicv3`、各类 PCIe 访问后端、UART、RISC-V/LoongArch/x86 相关开关等）整段移除。

**必要性**：避免两套真相（既在 `Cargo.toml` 又在 Kconfig）；编译期开关改由 `build.rs` 根据 `.config` 注入 `rustc --cfg`，Cargo 不再参与 feature 组合。

### 2.2 `Makefile`

新增/调整要点：

- **`kconfig_python`**：`tools/kconfig/.venv/bin/python`（本地 venv 中的 kconfiglib）。
- **`defconfig` / `menuconfig` / `savedefconfig`**：调用 Python 脚本或 shell 维护 `.config` 与板级 `defconfig`。
- **`ensure_config`**：根目录无 `.config` 时自动对该 `ARCH`/`BOARD` 执行 `defconfig`（需已 `bootstrap_venv.sh`）。
- **`all` 依赖**：从 `clean_check gen_cargo_config ...` 变为 `clean_check ensure_config gen_cargo_config ...`；摘要行用 **`DEFCONFIG`** 路径代替 **`FEATURES`**。
- **`elf` / `clippy` / `test` 等**：在合适处增加 `ensure_config`，保证 `cargo` 调用前已有 `.config`。
- **移除**：`FEATURES` 变量、`read_features.sh` 调用、`build_args` 中的 `--features "$(FEATURES)"`。

**必要性**：构建入口与 CI 一致地先物化 `.config`，再交给 `build.rs` 转成 `rustc` cfg。

### 2.3 `tools/read_features.sh` 删除

该脚本仅用于拼接 `cargo --features` 的输入。

**必要性**：功能被 **`kconfig_cli.py defconfig`** + 根 `.config` 取代，删除避免误用旧路径。

### 2.4 `tools/kconfig/host_config.sh`

合并原 **`tools/gen_cargo_config.sh`** 与 **`tools/gen_vscode_settings.sh`**：子命令 **`cargo`** 从根 **`.config`** 读取 `ARCH`/`BOARD`/`HVISOR_SRC`/`LD_SCRIPT`/`TEMPLATE` 并生成 **`.cargo/config.toml`**；子命令 **`vscode`** 先按当前 `ARCH`/`BOARD`（或 `BID`）调用 **`kconfig_cli.py defconfig`** 刷新 `.config`，再调用 **`kconfig_cli.py vscode-cfgs`** 写 **`.vscode/settings.json`**。脚本始终 `cd` 到仓库根，可从任意 cwd 调用。

**必要性**：与 Kconfig 相关的宿主机侧配置集中在一个目录；减少 `tools/` 顶层脚本数量。

**注意**：`cargo` 子命令须先存在 `.config`（例如 `make defconfig`）。

### 2.5 `tools/kconfig/merge_config.py` 删除

与 **`kconfig_cli.py defconfig` + 环境变量 `ARCH`/`BOARD`** 重复；需要命令行板型时可直接 **`ARCH=… BOARD=…`** 调用 **`kconfig_cli.py defconfig`**。

---

## 三、`build.rs` 行为（核心）

相对旧版，主要新增/强化逻辑：

1. **`load_cfg_map`**：解析 `kconfig/cfg_map.toml` 的 `[symbols]` 表。
2. **`parse_enabled_config_keys`**：扫描 `.config` 中值为 `y` 或 `m` 的 `CONFIG_*` 键。
3. 对每个映射到的 **Rust cfg 名** 打印 `cargo:rustc-check-cfg=cfg(...)`（配合 Rust 1.80+ 对自定义 cfg 的检查，减少拼写错误）。
4. 对每个启用的 `CONFIG_*`，若存在于映射表则 `cargo:rustc-cfg=<rust_cfg>`；未知 `CONFIG_*` 写日志警告。
5. **`rerun-if-changed`**：除原 `.config` 外，增加 `cfg_map.toml`、`Kconfig`、`tools/kconfig/kconfig_cli.py`、对应板 `kconfig/defconfig`、`board.rs` 等，保证配置变更会触发重新构建脚本。

仍保留：从 `.config` 解析 `ARCH`/`BOARD`/`BID`、`__board.rs` 符号链接、`target/build_rs.log` 日志。

**必要性**：在 **不修改 `Cargo.toml` features** 的前提下，把 Kconfig 输出接到 Rust 的条件编译；映射表集中管理，避免在 `build.rs` 里硬编码长列表。

---

## 四、Kconfig 与映射文件

### 4.1 `kconfig/Kconfig`

定义与旧 feature 集对应的 `config` 符号（如 `IRQ_GICV2`/`IRQ_GICV3`、`PCI`、`PCIE_*`、`IOMMU` 及各硬件后端、`PL011`、`AIA`、`GRAPHICS` 等），并使用 `depends on` / `select` 表达依赖（例如 `SHARE_S2PT` `select IOMMU`，PCIe 子选项 `depends on PCI`）。

**必要性**：提供可校验的配置空间；`kconfig_cli.py defconfig` / `menuconfig` 加载 defconfig 时会应用这些规则。

### 4.2 `kconfig/cfg_map.toml`

键为 `.config` 中的 **`CONFIG_*` 字面量**，值为源码中使用的 **自定义 cfg 标识符**（如 `CONFIG_IRQ_GICV3` → `irq_gicv3`）。

**必要性**：Kconfig 符号名习惯全大写加前缀，而 Rust `cfg` 惯用小写蛇形；单层映射表避免二者混用导致难以检索。

**命名调整说明（相对旧 Cargo feature）**：ARM GIC 由原先的 feature 名 `gicv2` / `gicv3` 改为 **`irq_gicv2` / `irq_gicv3`**（与 `CONFIG_IRQ_GICV2` 等对齐），源码中所有 `#[cfg(feature = "gicv3")]` 类用法改为 `#[cfg(irq_gicv3)]` 等。其余多数 cfg 名与旧 feature 名一致或仅增加 `CONFIG_` 前缀映射。

### 4.3 各板 `platform/.../kconfig/defconfig`

由迁移脚本或手工从原 `cargo/features` 生成；内容为 `CONFIG_*=y` 行（排序便于 diff）。

**必要性**：板级默认配置仍与旧行为对齐，但格式与内核 defconfig 一致，便于 review。

部分 `platform.mk` 等若曾引用旧路径，会改为指向 `kconfig/defconfig` 或相关说明（以具体板文件为准）。

---

## 五、Python / Shell 工具链（新增代码含义）

### 5.1 `tools/kconfig/requirements.txt` + `vendor/*.whl`

依赖 **kconfiglib**（及 **PySocks**，供代理场景）。`bootstrap_venv.sh` 使用 **`--no-index --find-links=vendor`** 安装，避免在仅 `socks5://` 代理环境下首次 `pip install` 因缺少 PySocks 失败（Jenkins 常见）。

**必要性**：可复现、可离线的工具链引导；与 [`kconfig.md`](./kconfig.md) 中说明一致。

### 5.2 `tools/kconfig/bootstrap_venv.sh`

创建 `tools/kconfig/.venv` 并从 `vendor/` 安装 wheel。

**必要性**：统一团队与 CI 的 Python 环境；`.gitignore` 忽略 `.venv/`，避免把虚拟环境提交进 Git。

### 5.3 `tools/kconfig/kconfig_cli.py`

统一入口（`argparse` 子命令）：

- **`defconfig`**：读环境变量 **`ARCH`/`BOARD`**，加载 **`kconfig/Kconfig`** 与 **`platform/.../kconfig/defconfig`**，校验 **Kconfig 中四选一架构** 与路径一致后，写根目录 **`.config`**（Kconfig 主体 + 元数据 `ARCH`/`BOARD`/`BID`/`HVISOR_SRC`/`LD_SCRIPT`/`TEMPLATE`）。
- **`menuconfig`**：终端 UI；保存后同样写回带元数据的 **`.config`**。仓库内**不要**再增加名为 **`menuconfig.py`** 的脚本，以免遮蔽 kconfiglib 的同名模块。
- **`vscode-cfgs <.config> <cfg_map.toml>`**：输出 **rust-analyzer** 用的 **`rustc` cfg** JSON 数组（`tomllib` + 映射表）。

**必要性**：减少分散的 Python 小文件，便于维护与 `build.rs` 的 `rerun-if-changed` 指向单一脚本。

### 5.4 `tools/kconfig/save_defconfig.sh`

从根 `.config` 中 `grep '^CONFIG_'` 排序写回 `platform/$ARCH/$BOARD/kconfig/defconfig`。

**必要性**：与内核 `savedefconfig` 工作流类似，便于把交互或本地调试结果固化进仓库。

### 5.5 `tools/kconfig/bootstrap_defconfigs.py`

遍历仍存在的 `platform/*/.../cargo/features`，通过内置 **`FEATURE_LINES`** 映射表把旧 feature **token** 转为 `CONFIG_*=y`，写入对应 `kconfig/defconfig`；并为目录名对应的架构追加 **`CONFIG_ARCH_*=y`**。PCIe 后端默认值由 **`kconfig/Kconfig`** 中 `choice` 的 **`default PCIE_ECAM`** 在后续 **`kconfig_cli.py defconfig`** 时处理。

**必要性**：一次性从旧目录结构批量生成新 defconfig，降低手工迁移错误率。

---

## 六、Rust 源码层面的改动模式

### 6.1 条件编译属性

- **由** `#[cfg(feature = "xxx")]` **改为** `#[cfg(xxx)]`（`xxx` 为 `cfg_map.toml` 右侧字符串）。
- **`cfg(feature = ...)` 与 `cfg(all(...))`**：凡与平台驱动相关的，一律改为自定义 cfg 名（与 `build.rs` 输出一致）。

### 6.2 `src/pci/mod.rs` 等处的 `compile_error!`

在原先互斥依赖人工约束的基础上，用 **`#[cfg(all(ecam_pcie, dwc_pcie))]`** 等组合在编译期直接报错（与 Kconfig 的互斥可双重保险；Kconfig 侧若未完全禁止组合，Rust 侧仍可兜底）。

**必要性**：PCIe 访问后端多选一，错误组合应尽早失败。

### 6.3 其它模块（`src/device/*`、`src/arch/*`、`src/zone.rs`、`src/logging.rs` 等）

同一模式：按板能力用 `#[cfg(pci)]`、`#[cfg(iommu)]`、`#[cfg(irq_gicv3)]` 等包裹模块、impl 或代码路径。

**必要性**：语义与旧 feature 一致，仅配置来源改变。

### 6.4 `src/main.rs` / `src/platform/mod.rs`

确保启动路径读取的板级信息与新的配置流一致（具体 diff 以该提交为准，通常为 cfg 名或条件分支对齐）。

---

## 七、IDE、CI 与 x86 板级小改

### 7.1 `tools/kconfig/host_config.sh`（`vscode`）

在生成 `settings.json` 前调用 **`kconfig_cli.py defconfig`** 刷新 `.config`，再用 **`kconfig_cli.py vscode-cfgs`** 生成 `rust-analyzer.cargo.cfgs`（`make vscode`）。

**必要性**：保证编辑器与当前 `ARCH`/`BOARD` 及 Kconfig 展开结果一致。

### 7.2 GitHub Actions / `Jenkinsfile`

- **GitHub Actions**：该提交中 **仅修改一条注释**，说明板级脚本与 **`kconfig/defconfig`** 的关系（并指向 `docs/kconfig.md`）；矩阵与安装步骤未变。本地/CI 要成功执行 **`make defconfig` / `ensure_config`**，仍需可用的 **`tools/kconfig/.venv`**（见 `bootstrap_venv.sh`）；是否在 GHA 各 job 里显式引导 venv 以当前仓库 workflow 为准。
- **Jenkinsfile**：矩阵构建单元内增加 **`bootstrap_venv.sh`** + **`make defconfig ARCH=... BOARD=...`**，日志中明确不再使用 **`cargo --features`**。

**必要性**：Jenkins 在干净工作区中先创建 venv 再生成 `.config`，与「根 `.config` 不入库」模型一致。

### 7.3 `platform/x86_64/*/board.rs`

个别板文件小改（通常与条件编译或默认配置引用路径对齐，见该提交 diff）。

---

## 八、文档与忽略规则

- **`docs/kconfig.md`**：新增，描述命令、目录布局、`menuconfig` 常见问题、与 `rust-analyzer` 的关系、Jenkins 行为等（用户日常查阅）。
- **`.gitignore`**：增加 `tools/kconfig/.venv/`、`tools/kconfig/__pycache__/` 等，避免将本地 Python 环境提交入库。

---

## 九、提交中带有的 `kernel_build/` 目录说明

该 commit 的统计中包含 `kernel_build/` 下的 `pyvenv.cfg`、指向 Python 的 symlink 等。通常 **虚拟环境不应入库**；若仓库后续已删除或 `.gitignore` 已屏蔽，以当前主干为准。若仍保留，建议团队确认是否为误提交或是否有独立用途（一般推荐仅保留 `tools/kconfig/.venv` 由脚本本地生成）。

---

## 十、文件改动分类速查（按职责）

| 类别 | 路径示例 |
|------|-----------|
| Kconfig 定义 | `kconfig/Kconfig` |
| 映射表 | `kconfig/cfg_map.toml` |
| 板级默认 | `platform/**/kconfig/defconfig` |
| 构建脚本 | `Makefile`, `build.rs`, `tools/kconfig/host_config.sh` |
| Kconfig 工具 | `tools/kconfig/kconfig_cli.py`、`bootstrap_venv.sh`、`save_defconfig.sh`、`host_config.sh`、`requirements.txt`、`vendor/*.whl` |
| 迁移辅助 | `tools/kconfig/bootstrap_defconfigs.py` |
| 源码 cfg 切换 | `src/**/*.rs`（属性由 `feature` 改为自定义 cfg） |
| 包定义 | `Cargo.toml`（移除 `[features]`） |
| 文档 | `docs/kconfig.md`、本文档 |

---

## 十一、小结

| 问题 | 答案 |
|------|------|
| 配置真相在哪里？ | 板级 `kconfig/defconfig` + `kconfig/Kconfig` 规则；生成根 `.config`；`cfg_map.toml` 定义到 Rust cfg 的映射。 |
| Cargo 还参与开关吗？ | 不参与；`build.rs` 根据 `.config` 注入 `rustc-cfg`。 |
| 为何引入 Python？ | 复用 **kconfiglib** 做 `defconfig`/`menuconfig` 与依赖展开，与手写 parser 相比更可靠。 |
| 旧 `cargo/features` 怎么办？ | 可用 `bootstrap_defconfigs.py` 批量转换；长期以 `kconfig/defconfig` 为准。 |

若仅需操作步骤与故障排除，请优先阅读 [`kconfig.md`](./kconfig.md)。
