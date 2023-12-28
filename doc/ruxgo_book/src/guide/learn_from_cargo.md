# 借鉴 Cargo

Rust 1.8 发布以后，Rust 团队便放弃了 Unix 系统传统的 [Make](https://www.gnu.org/software/make/) 工具，转而使用他们自己的 [Cargo 包管理工具](https://github.com/rust-lang/cargo)。当然，为了实现自举，减少对外部工具的依赖，Rust 必须通过自己的语言构建一些工具。其他语言也大多都经过这个过程。Google 的 Go 语言，从1.5版本开始，其编译器和解释器都由 Go 语言实现（有一小部分用了汇编），放弃了基于 C 语言的工具。

从 Make 换到 Cargo 的[原因](https://github.com/rust-lang/rust/pull/31123)，Rust团队核心成员 *Alex Crichton* 解释道：

> “在这个星球上只有一小部分人能够熟练使用 Makefile，这意味着对构建系统的贡献几乎是不存在，而且如果需要对构建系统进行更改，它是很难弄清楚应该怎么做。这种障碍使我们无法在 make 中做一些也许更花哨的事情”；
>
> “Make虽然可移植，但不幸的是，它并不是无限可移植的。例如，最近引入的MSVC目标不太可能默认安装（例如, 它目前需要在MSYS2 shell内构建）。相反，make的可移植性是以疯狂和奇怪的操作为代价的，需要围绕各种软件版本进行工作，特别是在配置脚本和 Makefile 方面。”
>
> “改变编译系统使 Rust 标准库和编译器可以轻松从crates.io生态系统中获益。”	
>
> ……		
