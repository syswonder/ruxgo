# Ruxgo 安装

要安装`ruxgo`可执行文件，您首先需要安装 Rust 和 Cargo。按照[Rust安装页面](https://www.rust-lang.org/tools/install)上的说明操作。Ruxgo 目前至少需要 Rust 1.74 版本。

一旦您安装了 Rust，就可以使用以下命令来构建和安装 Ruxgo:

```sh
cargo install ruxgo
```

这将自动从[crates.io](https://crates.io/)下载并构建 Ruxgo，并将其安装到 Cargo 的全局二进制目录（默认为`~/.cargo/bin/`）。

发布到 crates.io 的版本将稍微落后于 GitHub 上托管的版本。如果您需要最新版本，您可以自己构建 Ruxgo 的 git 版本：

```sh
cargo install --git https://github.com/syswonder/ruxgo.git ruxgo
```

**注意:**

如果您在安装时遇到问题，可能需要安装一些构建依赖项，请参考 RuxOS 下的[README.md](https://github.com/syswonder/ruxos?tab=readme-ov-file#install-build-dependencies).