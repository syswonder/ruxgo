# ruxgo init

当您开始一个新项目时，通常需要一些基本的框架设置。`ruxgo` 提供了 `init` 命令来帮助您快速开始一个新的 C 或 C++ 项目。

## 使用方式

要初始化一个新项目，使用以下命令：

```
ruxgo init <项目名称> [--c] [--cpp]
```

- `<项目名称>`: 指定新项目的名称。
- `--c`: 初始化一个 C 语言项目。
- `--cpp`: 初始化一个 C++ 语言项目。

注意：`--c` 和 `--cpp` 选项不能同时使用。如果都不指定，则默认创建一个 C++ 项目。

## 命令行为

当运行 `ruxgo init` 命令后，将会在当前目录下创建一个新的项目目录，其中包括以下内容：

```
<项目名称>/
├── src/
│   ├── main.c 或 main.cpp
│   └── include/
├── .gitignore
├── README.md
├── LICENSE
└── config_<platform>.toml
```

- `src/`：包含所有源代码文件和配置文件。
- `src/include`：用于存放头文件。
- `.gitignore`：用于指定 git 忽略的文件和目录。
- `README.md` 和 `LICENSE`：项目的基本文档。
- `config_<platform>.toml`：项目的配置文件，根据运行平台的不同而变化（例如 `config_linux.toml` 或 `config_win32.toml`）。

## 示例

- 初始化一个名为 "my_project" 的 C++ 项目：

  ```
  ruxgo init my_project --cpp
  ```

- 初始化一个名为 "my_project" 的 C 项目：

  ```
  ruxgo init my_project --c
  ```

## 提示

- 使用 `--help` 选项可以查看更多命令帮助。
- 当项目目录已经存在时，`ruxgo init` 将不会继续执行，并显示错误信息。
- 通过修改 `config_<platform>.toml` 文件，您可以自定义编译器选项和其他构建设置。