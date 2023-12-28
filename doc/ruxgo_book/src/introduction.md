# 简介

[Ruxgo](https://github.com/syswonder/ruxgo) 是一个类似 Cargo 的构建工具，用于构建 C 和 C++ 应用程序，它只依赖于一个 Toml 文件。它摒弃了原有 MAKE 构建工具中的复杂语法和依赖规则构造的方式，展现了最原始的编译过程。如果你讨厌使用 Makefile，不妨尝试使用 Ruxgo 来构建应用程序，只需花几分钟!

如果你想要构建一个项目，你只需要弄清楚源文件路径、头文件路径、cflags 和 ldflags，然后将它们填写到 Toml 文件中的对应位置。剩下的都是 Ruxgo 做，很简单!