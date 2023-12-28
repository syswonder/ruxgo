# TOML 文件示例

下面是 [sqlite3](https://github.com/Ybeichen/ruxgo/tree/master/apps/sqlite3) 的 Toml 文件示例，具体运行可参考 `apps/sqlite3` 下的 README.md 。

带有库和可执行文件的示例文件（在本地运行）:

```toml
[build]
compiler = "gcc"

[[targets]]
name = "libsqlite3"
src = "./sqlite-amalgamation-3410100"
src_excluded = ["shell.c"]
include_dir = "./sqlite-amalgamation-3410100"
type = "static"
cflags = "-w -DSQLITE_THREADSAFE=0 -DSQLITE_OMIT_FLOATING_POINT -DSQLITE_OMIT_LOAD_EXTENSION -DSQLITE_DEBUG"
archive = "ar"
ldflags = "rcs"

[[targets]]
name = "local_sqlite3"
src = "./"
src_excluded = ["sqlite-amalgamation-3410100"]
include_dir = "./"
type = "exe"
cflags = ""
ldflags = ""
deps = ["libsqlite3"]
```

带有库和可执行文件的示例文件（在ruxos上运行）:

```toml
[build]
compiler = "gcc"

[os]
name = "ruxos"
services = ["fp_simd","alloc","paging","fs","blkfs"]
ulib = "ruxlibc"

[os.platform]
name = "x86_64-qemu-q35"
smp = "4"
mode = "release"
log = "error"

[os.platform.qemu]
blk = "y"
graphic = "n"

[[targets]]
name = "libsqlite3"
src = "./sqlite-amalgamation-3410100"
src_excluded = ["shell.c"]
include_dir = "./sqlite-amalgamation-3410100"
type = "static"
cflags = "-w -DSQLITE_THREADSAFE=0 -DSQLITE_OMIT_FLOATING_POINT -DSQLITE_OMIT_LOAD_EXTENSION -DSQLITE_DEBUG"
archive = "ar"
ldflags = "rcs"

[[targets]]
name = "ruxos_sqlite3"
src = "./"
src_excluded = ["sqlite-amalgamation-3410100"]
include_dir = "./"
type = "exe"
cflags = ""
linker = "rust-lld -flavor gnu"
ldflags = ""
deps = ["libsqlite3"]
```