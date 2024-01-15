# TOML 文件说明

由于 C/C++ 项目需要处理复杂的项目配置，包括库依赖、编译选项、链接选项等，TOML 文件格式提供了一个清晰、结构化且易于理解的方式来组织这些信息。它使得开发者可以轻松地查看和编辑项目设置，同时确保这些设置的精确性和一致性。

Ruxgo 采用 `config_<platform>.toml` 文件来配置各个项目（`<platform>`根据运行平台的不同而变化，例如 `config_linux.toml` 或 `config_win32.toml`）。此外，由于 TOML 的跨平台和语言无关特性，ruxgo 可以更容易地集成到不同的环境和工作流中，提高了其适应性和可扩展性。

`config_<platform>.toml` 文件由一个 **[build]** 模块和多个 **[targets]** 模块组成。如果你想在 RuxOS 上运行，可以添加 **[os]** 模块。

* [builder 模块](./builder_module.md)

* [target 模块](./target_module.md)

* [os 模块](./os_module.md)