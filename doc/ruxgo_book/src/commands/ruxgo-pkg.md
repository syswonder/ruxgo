# ruxgo pkg

`ruxgo pkg` 命令用于管理软件包，包括列出、拉取、更新和清理软件包等。

## 使用方式

要使用 `ruxgo pkg` 命令，您可以执行以下操作：

```
ruxgo pkg [选项]
```

可选项如下：

- `-l, --list`: 列出远程仓库中可用的软件包。
- `-p, --pull <PKG_NAME>`: 从远程仓库拉取特定软件包。
- `-r, --run <APP_BIN>`: 运行特定的应用程序二进制文件。
- `-u, --update <PKG_NAME>`: 更新特定软件包。
- `-c, --clean <PKG_NAME>`: 清理特定软件包。
- `--clean-all`: 清理所有软件包。
- `-h, --help`: 打印帮助信息。

下载的文件保存在 `ruxgo_pkg/` 目录中，具体内容如下：

- `app-bin/`: 存放 app 可执行文件及其对应脚本。
- `ruxos/`: ruxos 存储位置。
- `rux-*/`: app 源码存储位置。
- `cache/`: 存放 packages 信息的缓存。

## 示例

- 列出所有可用的软件包：

  ```
  ruxgo pkg --list
  ```

- 从远程仓库拉取名为 "example_pkg" 的软件包：

  ```
  ruxgo pkg --pull example_pkg
  ```

- 更新名为 "example_pkg" 的软件包：

  ```
  ruxgo pkg --update example_pkg
  ```

- 清理名为 "example_pkg" 的软件包：

  ```
  ruxgo pkg --clean example_pkg
  ```

- 清理所有软件包：

  ```
  ruxgo pkg --clean-all
  ```

## 提示

- 使用 `--help` 选项可以查看更多命令帮助。
- 确保在执行任何软件包管理操作之前，您的系统已连接到互联网。