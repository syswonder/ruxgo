# builder 模块

**[build]** 模块描述了编译器的类型和所需远程库包。它包含两个部分: `compiler` 和 `packages`。

- `compiler`: 指定编译器类型，例如: "gcc"。

- `packages`: 可选。主要用于从 Github 中获取应用的源代码，然后通过解析其中的 `config_linux.toml`文件来获取所需的库。当使用包时，你需要指定远程仓库和分支。