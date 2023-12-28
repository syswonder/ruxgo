# os 模块

**[os]** 模块是可选的。如果你想在本地运行，[build] 和 [targets] 是完全满足，如果你想在 RuxOS 上运行，则需要添加[os] 模块。

添加 [os] 模块后，原有 [targets] 模块的一些内容将会发生改变。例如:

当 [os] 模块的平台是 "x86_64-qemu-q35" 时，编译器类型不再是 "gcc"，它将变成 "x86_64-linux-musl-gcc"。此外，所有 [targets] 的 cflags 都默认添加了 "-nostdinc -fno-builtin -ffreestanding -Wall"，当 [targets] 类型是 "exe" 时，ldflags 还默认添加了 "-nostdlib -static -no-pie --gc-sections"，你不需要去手动添加它们。当然，根据架构和平台的不同，还有其他默认的添加及改变。

Ruxgo 通过修改一些代码逻辑实现，这么做是为了，如果你能在本地跑通一个程序，那么只需要将定制的 [os] 模块拼接到你本地跑通的模块上，即可实现在 RuxOS 上流畅的运行，而不需要额外的操作!

具体 **[os]** 模块描述如下:

- `name`: 指定操作系统的名称。

- `services`: 指定操作系统可以提供的服务，类似于 RuxOS 中的 `features`。

- `ulib`: 指定想要使用的用户库，可选项有: "ruxlibc"，"ruxmusl"。

- `platform`: 如果需要，请在 [os.platform] 中进行配置。

如果你想进一步配置平台，可以在 **[os.platform]** 中实现。如果为空，则使用默认值。具体细节如下:

- `name`: 指定操作系统在哪个平台上运行，可选项有: "x86_64-qemu-q35"， "aarch64-qemu-virt"， "riscv64-qemu-virt"。默认值为 "x86_64-qemu-q35"。

- `smp`: 指定cpu数量。默认值为 "1"。

- `mode`: 指定构建模式，可选项有: "release"，"debug"。默认值为 "release"。

- `log`: 指定日志级别，可选项有: "warn"，"error"，"info"，"debug" 和 "trace"。默认值为 "warn"。

- `v`: 指定 verbose 级别，可选项有: ""，"1"，"2"。默认值为 ""。

- `qemu`: 如果需要，请在 [os.platform.qemu] 中进行配置。

如果你的平台依赖于 qemu ，你需要在 **[os.platform.qemu]** 中进一步配置它。如果为空，则使用默认值。具体细节如下:

- `blk`: 指定是否启用存储设备（virtio-blk）。默认值为 "n"。

- `net`: 指定是否启用网络设备（virtio-net）。默认值为 "n"。

- `graphic`: 指定是否启用显示设备和图形输出（virtio-gpu）。默认值为"n"。

- `disk_img`: 指定虚拟磁盘镜像的路径。默认值为 "./disk_img"。

- `v9p`: 指定是否启用 virtio-9p 设备。默认值为 "n"。

- `v9p_path`: 指定 virtio-9p 后端的主机路径。默认值为 "./"。

- `qemu_log`: 指定是否启用 QEMU 日志（日志文件为 "qemu.log" ）。默认值为 "n"。

- `net_dump`: 指定是否启用网络包转储（日志文件为 "netdump.pcap" ）。默认值为 "n"。

- `net_dev`: 指定 QEMU 网络设备后端类型: "user" 或 "tap"。默认值为 "user"。

- `ip`: 指定 IPv4 地址。QEMU "user" 网络设备的默认值为 "10.0.2.15"。

- `gw`: 指定 IPv4 地址的网关。QEMU "user" 网络设备的默认值为 "10.0.2.2"。

- `args`: 指定命令行参数，以逗号分隔。它用于传递特定的变量，如`argc`、`argv`。默认值为 ""。

- `envs`: 指定环境变量，键值对之间用逗号分隔。默认值为 ""。