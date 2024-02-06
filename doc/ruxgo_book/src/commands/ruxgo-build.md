# ruxgo -b

`ruxgo -b` 命令用于构建您的项目，需确保当前目录下存在 `config_<platform>.toml`。

## 使用方式

要构建当前项目，您可以执行以下操作：

```bash
ruxgo -b [--path <路径>] [--gen-cc] [--gen-vsc]
```

- `--path <路径>`: 指定一个特定的目录（需存在 `config_<platform>.toml`）来执行构建操作。如果不提供，则默认在当前目录下执行。
- `--gen-cc`: 生成 `compile_commands.json` 文件，它包含了编译项目的所有命令。
- `--gen-vsc`: 生成 Visual Studio Code 的配置文件 `.vscode/c_cpp_properties.json`，它包含了项目的编译器配置和头文件路径。

## 命令行为

当执行 `ruxgo -b` 命令后，将会在当前目录下创建一个名为 `ruxgo_bld/` 的构建目录，包括以下内容：

```bash
ruxgo_bld/
├── bin/
├── obj_linux/ 或 obj_win32/
├── target/
├── *.hash
├── compile_commands.json (如果启用了gen_cc)
├── .vscode/c_cpp_properties.json (如果启用了gen_vsc)
└── ruxmusl/ (如果使用了ruxmusl用户库)
```

- `bin/`： 存放构建过程中生成的静态库、动态库、目标文件或可执行文件 。
- `obj_linux/obj_win32`： 存放编译源码生成的中间对象文件 （ *.o ）。
- `target`： 存放构建 ruxos 后生成的 target 文件。
- `*.hash`： 存放构建过程中生成的 hash 文件，用来实现增量构建。
- `compile_commands.json`： 存放构建过程中的所有编译命令，如果启用了 gen_cc。
- `.vscode/c_cpp_properties.json`： 存放项目的 vscode 配置，如果启用了 gen_vsc。
- `ruxmusl/`： 存放构建 ruxmusl 后生成的中间文件及静态库，如果使用了 ruxmusl 。