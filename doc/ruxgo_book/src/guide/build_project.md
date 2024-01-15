# 构建一个项目

首先，确保您的系统上已安装 ruxgo。如果尚未安装，请参考 [Ruxgo 安装](../installation.md) 。

之后使用如下命令初始化一个 C 项目：

```bash
ruxgo init my_project --c
```

初始化完成后，切换到 my_project 目录，使用 ruxgo 构建并运行：

```bash
ruxgo -b
ruxgo -r
```

如果您想在 RuxOS 上运行，将 my_project 下的`config_<platform>.toml`文件修改为如下:

```toml
[build]
compiler = "gcc"

[os]
name = "ruxos"
services = []
ulib = "ruxlibc"

[os.platform]
name = "x86_64-qemu-q35"
mode = "release"
log = "info"

[os.platform.qemu]
graphic = "n"

[[targets]]
name = "main"
src = "./src/"
include_dir = "./src/include/"
type = "exe"
cflags = "-g -Wall -Wextra"
linker = "rust-lld -flavor gnu"
ldflags = ""
deps = []
```

之后将 my_project 复制到 ruxos/apps/c 目录下，切换到 my_project 目录，使用 ruxgo 构建并运行：

```bash
ruxgo -b
ruxgo -r
```
