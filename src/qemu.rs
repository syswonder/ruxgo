use crate::utils::{PlatformConfig, log, LogLevel};
use crate::builder::Target;

pub fn config_qemu(platform_config: &PlatformConfig, trgt: &Target) -> (Vec<String>, Vec<String>) {
    // vdev_suffix
    let vdev_suffix = match platform_config.qemu.bus.as_str() {
        "mmio" => "device",
        "pci" => "pci",
        _ => {
            log(LogLevel::Error, "BUS must be one of 'mmio' or 'pci'"); 
            std::process::exit(1);
        }
    };
    // config qemu
    let mut qemu_args = Vec::new();
    qemu_args.push(format!("qemu-system-{}", platform_config.arch));
    // init
    qemu_args.push("-m".to_string());
    qemu_args.push("128M".to_string());
    qemu_args.push("-smp".to_string());
    qemu_args.push(platform_config.smp.clone());
    // arch
    match platform_config.arch.as_str() {
        "x86_64" => {
            qemu_args.extend(
                vec!["-machine", "q35", "-kernel", &trgt.elf_path]
                .iter().map(|&arg| arg.to_string()));
        }
        "risc64" => {
            qemu_args.extend(
                vec!["-machine", "virt", "-bios", "default", "-kernel", &trgt.bin_path]
                .iter().map(|&arg| arg.to_string()));
        }
        "aarch64" => {
            qemu_args.extend(
                vec!["-cpu", "cortex-a72", "-machine", "virt", "-kernel", &trgt.bin_path]
                .iter().map(|&arg| arg.to_string()));
        }
        _ => {
            log(LogLevel::Error, "Unsupported architecture");
            std::process::exit(1);
        }
    };
    // blk
    if platform_config.qemu.blk == "y" {
        qemu_args.push("-device".to_string());
        qemu_args.push(format!("virtio-blk-{},drive=disk0", vdev_suffix));
        qemu_args.push("-drive".to_string());
        qemu_args.push(format!("id=disk0,if=none,format=raw,file={}", platform_config.qemu.disk_img));
    }
    // net
    if platform_config.qemu.net == "y" {
        qemu_args.push("-device".to_string());
        qemu_args.push(format!("virtio-net-{},netdev=net0", vdev_suffix));
        // net_dev
        if platform_config.qemu.net_dev == "user" {
            qemu_args.push("-netdev".to_string());
            qemu_args.push("user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555".to_string());
        } else if platform_config.qemu.net_dev == "tap" {
            qemu_args.push("-netdev".to_string());
            qemu_args.push("tap,id=net0,ifname=tap0,script=no,downscript=no".to_string());
        } else {
            log(LogLevel::Error, "NET_DEV must be one of 'user' or 'tap'"); 
            std::process::exit(1);
        }
        // net_dump
        if platform_config.qemu.net_dump == "y" {
            qemu_args.push("-object".to_string());
            qemu_args.push("filter-dump,id=dump0,netdev=net0,file=netdump.pcap".to_string());
        }
    }

    // graphic
    if platform_config.qemu.graphic == "y" {
        qemu_args.push("-device".to_string());
        qemu_args.push(format!("virtio-gpu-{}", vdev_suffix));
        qemu_args.push("-vga".to_string());
        qemu_args.push("none".to_string());
        qemu_args.push("-serial".to_string());
        qemu_args.push("mon:stdio".to_string());
    } else if platform_config.qemu.graphic == "n" {
        qemu_args.push("-nographic".to_string());
    }
    // qemu_log
    if platform_config.qemu.qemu_log == "y" {
        qemu_args.push("-D".to_string());
        qemu_args.push("qemu.log".to_string());
        qemu_args.push("-d".to_string());
        qemu_args.push("in_asm,int,mmu,pcall,cpu_reset,guest_errors".to_string());
    }
    // debug
    let mut qemu_args_debug = Vec::new();
    qemu_args_debug.extend(qemu_args.clone());
    qemu_args_debug.push("-s".to_string());
    qemu_args_debug.push("-S".to_string());
    // acceel
    if platform_config.accel == "y" {
        if cfg!(target_os = "darwin") {
            qemu_args.push("-cpu".to_string());
            qemu_args.push("host".to_string());
            qemu_args.push("-accel".to_string());
            qemu_args.push("hvf".to_string());
        } else {
            qemu_args.push("-cpu".to_string());
            qemu_args.push("host".to_string());
            qemu_args.push("-accel".to_string());
            qemu_args.push("kvm".to_string()); 
        } 
    }

    (qemu_args, qemu_args_debug)
}
    