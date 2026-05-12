# 平台配置（Kconfig / defconfig）

hvisor 的平台能力开关由 **Kconfig 符号**（写入根目录 `.config` 的 `CONFIG_*=y` 行）描述，构建时由 [`build.rs`](../build.rs) 读入并转换为 `rustc --cfg`（例如 `irq_gicv3`、`pci`）。源码中使用 `#[cfg(irq_gicv3)]` 等形式，**不再**使用 Cargo 的 `feature`。

## 根目录 `.config` 是怎么来的（流程与为何「多段拼接」）

1. **入口**：`make defconfig` 或 `make menuconfig` 结束时，在仓库根需要一份 **`.config`**，供 `cargo`/`build.rs` 与其它脚本使用。
2. **板级输入**：只读 **`platform/<ARCH>/<BOARD>/kconfig/defconfig`**。这里的 **`ARCH`/`BOARD` 来自 Makefile 导出的环境变量**（或手动设置后调用 **`kconfig_cli.py defconfig`**），用来**定位文件路径**；不要求把 `ARCH=` 写进 defconfig 文件本身。
3. **为何不把 `ARCH`/`BOARD` 写进 defconfig？**
   - defconfig 交给 **kconfiglib** 解析，内容应是 **`CONFIG_*=y`**（以及 `#` 注释），与 Linux 习惯一致；`ARCH=` 不是 Kconfig 符号，混进去要么要在加载前剥离，要么污染 `menuconfig`/`savedefconfig` 的语义。
   - 目录 **`platform/<arch>/<board>/`** 已经唯一确定了板型，再在 defconfig 里重复一遍容易与路径不同步。
4. **拼接内容**：[`tools/kconfig/kconfig_cli.py`](../tools/kconfig/kconfig_cli.py) 在写出 Kconfig 主体后，在文件**开头**追加几行 **Makefile/脚本 约定的元数据**（`ARCH`、`BOARD`、`BID`、`HVISOR_SRC`、`LD_SCRIPT`、`TEMPLATE` 的绝对路径）。这样一份文件同时满足：
   - **kconfiglib**：主体仍是标准 `CONFIG_*`；
   - **`build.rs`**：读 `ARCH`/`BOARD` 做 `__board.rs` 链接；
   - **[`tools/kconfig/host_config.sh`](../tools/kconfig/host_config.sh)**（子命令 `cargo`）：从同一份 `.config` 读取上述路径生成 `.cargo/config.toml`，**不再**根据环境变量重复拼 `platform/$ARCH/$BOARD/...`（避免两处维护）。

**一句话**：defconfig 只管 Kconfig；「哪块板」由**路径 + 调用参数**决定；根 `.config` 是 **元数据 + Kconfig 展开结果** 的合并文件，减少脚本重复逻辑。

## 常用命令

- **生成当前 ARCH/BOARD 的 `.config`**（合并元数据 + `CONFIG_*`）：

  ```bash
  make defconfig
  # 或（需已创建 venv 并安装 kconfiglib）
  ARCH=<arch> BOARD=<board> tools/kconfig/.venv/bin/python tools/kconfig/kconfig_cli.py defconfig
  ```

- **首次构建 / 切换板型**：根目录若无 `.config`，`make all` 会先 **`make defconfig`**（需 `tools/kconfig/.venv` + kconfiglib）。

- **交互菜单**（可选，需 Python venv + kconfiglib）：

  ```bash
  ./tools/kconfig/bootstrap_venv.sh
  make menuconfig
  ```

  （Jenkins 等仅提供 `socks5://` 代理时，在线 `pip install` 会因缺少 PySocks 报错；`bootstrap_venv.sh` 从 `tools/kconfig/vendor/*.whl` 离线安装，可绕过该问题。若升级 `requirements.txt` 中的版本，请同步更新 vendor 目录：`python3 -m pip download -d tools/kconfig/vendor -r tools/kconfig/requirements.txt`。）

- **`make menuconfig` 失败而 `make defconfig` 正常**  
  常见原因之一是曾将入口脚本命名为 `menuconfig.py`，与 kconfiglib 安装在 `site-packages` 里的 **`menuconfig` 模块同名**，`sys.path` 会优先加载仓库脚本，导致 `import menuconfig` 失败；入口已统一为 **`kconfig_cli.py menuconfig`**（不要新增名为 `menuconfig.py` 的仓库脚本）。若仍报错且含 `_curses` / `curses`，说明解释器缺少 curses 支持，请按发行版安装对应包（例如 Debian/Ubuntu 使用 **`apt install python3-curses`**，包名随 Python 主版本可能为 `python3.12-curses` 等；Fedora 常为 **`dnf install python3-curses`**）。

- **把当前 `.config` 中的 `CONFIG_*` 行写回板级 defconfig**：

  ```bash
  make savedefconfig
  ```

## 文件布局

| 路径 | 说明 |
|------|------|
| [`kconfig/Kconfig`](../kconfig/Kconfig) | 含 **Target architecture**（4 选 1）及驱动等菜单 |
| [`tools/kconfig/kconfig_cli.py`](../tools/kconfig/kconfig_cli.py) | **`defconfig`**：生成根 `.config`；**`menuconfig`**：交互配置；**`vscode-cfgs`**：输出 rust-analyzer 用的 cfg JSON（由 `host_config.sh` 调用） |
| [`tools/kconfig/host_config.sh`](../tools/kconfig/host_config.sh) | `cargo`：写 `.cargo/config.toml`；`vscode`：刷新 `.config` 并写 `.vscode/settings.json`（`make gen_cargo_config` / `make vscode`） |
| [`tools/kconfig/bootstrap_venv.sh`](../tools/kconfig/bootstrap_venv.sh) | 创建 `.venv` 并离线安装 kconfiglib |
| [`tools/kconfig/save_defconfig.sh`](../tools/kconfig/save_defconfig.sh) | `make savedefconfig`：把 `CONFIG_*` 写回板级 defconfig |
| [`kconfig/cfg_map.toml`](../kconfig/cfg_map.toml) | `CONFIG_*` → Rust `cfg` 名映射表（**唯一真相**） |
| `platform/<arch>/<board>/kconfig/defconfig` | 该板的默认 `CONFIG_*=y` 集合 |
| 根目录 `.config` | 本地生成，含 `ARCH`/`BOARD`/… 元数据 + Kconfig 输出（已被 `.gitignore`） |

## 仅运行 `cargo build` 时

须先在仓库根生成 `.config`（例如安装 kconfiglib venv 后 **`make defconfig`**），否则 `build.rs` 会因缺少 `ARCH`/`BOARD` 报错。

## 修改或新增开关

1. 在 `kconfig/Kconfig` 中增加 `config` 项（若需 `menuconfig` 校验/依赖）。**目标架构**在 **`Target architecture`** 菜单中为四选一；板级 **`kconfig/defconfig`** 里请**直接写上**与目录 **`platform/<arch>/`** 对应的一行 **`CONFIG_ARCH_*=y`**（与 Kconfig 一致即可，无需额外同步脚本）。
2. 在 `kconfig/cfg_map.toml` 的 `[symbols]` 中增加 `CONFIG_FOO = "rust_cfg_name"`。
3. 在 `build.rs` 中已通过映射表统一输出 `cargo:rustc-check-cfg`，一般**无需**再手写列表。
4. 更新相关板的 `kconfig/defconfig`（或运行 `make menuconfig` 后 `make savedefconfig`）。
5. 在 Rust 源码中使用 `#[cfg(rust_cfg_name)]`（完整列表见 [`rust-cfgs.md`](./rust-cfgs.md)）。

## rust-analyzer

运行 `make vscode` 会根据当前 `.config` 与 `cfg_map.toml` 生成 `.vscode/settings.json` 中的 `rust-analyzer.cargo.cfgs`，与真实编译开关对齐。

## 从旧版 `platform/.../cargo/features` 批量生成 defconfig

若调整了 `cargo/features` 行并希望同步到 `kconfig/defconfig`，可在仓库根执行：

```bash
python3 tools/kconfig/bootstrap_defconfigs.py
```

随后检查各板 `kconfig/defconfig` 并提交。

## Jenkins（`Jenkinsfile`）

矩阵编译阶段会先执行 **`make defconfig ARCH=… BOARD=…`**，再 **`make dtb`**（非 x86）、最后 **`make all MODE=release`**。对 **`jenkins/ci.yaml`** 中配置了测试的 BID，**Run test cases** 阶段在调用 **`jenkins/ci_runner.py`** 前会再次保证 **`tools/kconfig/.venv`** 存在（缺失则运行 **`bootstrap_venv.sh`**）并执行 **`make defconfig`**，以便后续 **`make ci-run`**（依赖 `ensure_config` / `host_config.sh`）与 Kconfig 一致。`.config` 不入库，每个 matrix 工作副本在编译前都会从对应板的 `kconfig/defconfig` 生成；与旧版通过 `FEATURES` / `cargo --features` 传参的方式已脱钩。
