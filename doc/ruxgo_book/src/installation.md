# Ruxgo 安装

要从源代码构建`ruxgo`可执行文件，你首先需要安装 Rust 和 Cargo。按照[Rust安装页面](https://www.rust-lang.org/tools/install)上的说明操作。Ruxgo 目前至少需要 Rust 1.74 版本。

一旦你安装了 Rust，就可以使用以下命令来构建和安装 Ruxgo:

```sh
cargo install --git https://github.com/syswonder/ruxgo.git ruxgo
```

这将自动下载、构建 Ruxgo，并将其安装到 Cargo 的全局二进制目录（默认为`~/.cargo/bin/`）。

要卸载，请执行命令`cargo uninstall ruxgo`。

**注意:**

如果你在安装时遇到问题，可能需要安装一些构建依赖项，请参考 RuxOS 下的[README.md](https://github.com/syswonder/ruxos?tab=readme-ov-file#install-build-dependencies).