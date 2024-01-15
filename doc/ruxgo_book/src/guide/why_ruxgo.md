# 为什么 Ruxgo

在深入探讨 Ruxgo 之前，有必要强调构建工具在任何软件开发项目中的重要性。构建工具不仅负责将源代码转换为可执行程序，还涉及依赖管理、自动化构建和测试、以及代码质量控制等关键环节。尤其对于 unikernel 操作系统，这些工具不仅需要支持标准的构建流程，还要适应 unikernel 的特殊要求，如高度的模块化、特定环境下的优化构建等。

[RuxOS](https://github.com/syswonder/ruxos) ，作为一个基于 Rust 语言开发的轻量级 unikernel 操作系统，面临着独特的挑战和需求。初期，RuxOS 采用了 Make 构建工具，依赖其灵活性和广泛的应用兼容性，成功地开发并移植了一些 C/C++ 应用程序。但随着项目的发展，Make 的一些局限性开始显现。例如:

- Makefile 的复杂性：随着新应用的不断移植，Makefile 的配置变得愈加繁琐，导致项目在维护和扩展方面面临挑战；
- Rust 生态集成不足：虽然 Makefile 可以封装 Cargo 命令来构建 RuxOS，但这种集成不是天然的，需要手动管理。这导致了 Rust 语言和 [crates.io](https://origin.eqing.tech/) 生态系统中的某些优势无法被充分利用；
- 构件化需求不匹配：RuxOS 遵循 unikernel 思想，旨在为单一应用生成可运行的二进制镜像。这要求构建工具能够提供组件管理、按需构建等高级功能，而 Make 在这方面存在限制。

因此，RuxOS 需要一个更加专业和高效的构建工具——这就是 Ruxgo 的起源。Ruxgo 是一个纯 Rust 编写的构建工具，与 RuxOS 完美融合。与其他 unikernel 操作系统类似，如 Unikraft 使用 Kraft，IncludeOS 使用 Conan，RuxOS 使用 Ruxgo 不仅解决了 Make 在现代软件开发中的局限性，还完全利用了 Rust 语言及其生态系统的优势。

相较于原有的 Make 构建工具，Ruxgo 具有以下**显著特点**：

- **简化的配置文件**：采用 [TOML](https://github.com/toml-lang/toml) 作为配置文件的格式，提供了一个语法简单、易于理解和维护的配置环境。使得开发者可以专注于构建逻辑，而非配置文件的复杂性；
- **利用 Rust 优势**：Ruxgo 一方面利用 Rust 本身的诸多优势特性，如内存安全、并发处理等。另一方面深度利用 RuxOS 和 [crates.io](https://crates.io/) 的生态系统，使得构建过程更加高效、流畅和稳定；
- **组件化构建功能**：针对 unikernel 和构件化操作系统的特点，Ruxgo 提供了模块化的构建方式以及包管理器，支持用户高效的按需编译和构件管理。