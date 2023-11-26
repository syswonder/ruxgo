use crate::utils::{PlatformConfig, log, LogLevel};
use crate::builder::Target;

pub fn config_qemu(platform_config: &PlatformConfig, trgt: &Target) -> Vec<String> {
    //let qemu = format!("qemu-system-{}", ARCH);
    // vdev_suffix
    let vdev_suffix = if platform_config.qemu.bus == "mmio" {
        "device"
    } else if platform_config.qemu.bus == "pci" {
        "pci"
    } else {
        log(LogLevel::Error, "BUS must be one of 'mmio' or 'pci'"); 
        std::process::exit(1);
    };

    let mut qemu_args_final = Vec::new();
    qemu_args_final.push("qemu-system-x86_64".to_string());

    // init
    let qemu_args_init = vec!["-m", "128M", "-smp"]
        .iter()
        .map(|&arg| arg.to_string())
        .collect::<Vec<String>>();
    qemu_args_final.extend(qemu_args_init);
    qemu_args_final.push(platform_config.smp.clone());
    // arch
    let qemu_args_x86_64 = vec!["-machine","q35","-kernel",&trgt.elf_path]
        .iter()
        .map(|&arg| arg.to_string())
        .collect::<Vec<String>>();
    let qemu_args_riscv64 = vec!["-machine","virt","-bios","default","-kernel",&trgt.bin_path]        
        .iter()
        .map(|&arg| arg.to_string())
        .collect::<Vec<String>>();
    let qemu_args_aarch64 = vec!["-cpu","cortex-a72","-machine","virt","-kernel",&trgt.bin_path]      
        .iter()
        .map(|&arg| arg.to_string())
        .collect::<Vec<String>>();
    if platform_config.arch == "x86_64" {
        qemu_args_final.extend(qemu_args_x86_64);
    } else if platform_config.arch == "risc64" {
        qemu_args_final.extend(qemu_args_riscv64);
    } else if platform_config.arch == "aarch64" {
        qemu_args_final.extend(qemu_args_aarch64);
    } else {
        log(LogLevel::Error, "Unsupported architecture"); 
        std::process::exit(1);
    };
    // blk
    let qemu_args_blk = vec![
            "-device".to_string(),
            format!("virtio-blk-{},drive=disk0", vdev_suffix),
            "-drive".to_string(),
            format!("id=disk0,if=none,format=raw,file={}", platform_config.qemu.disk_img),
        ];
    if platform_config.qemu.blk == "y" {
        qemu_args_final.extend(qemu_args_blk);
    }
    // net
    let qemu_args_net = vec![
            "-device".to_string(),
            format!("virtio-net-{}", vdev_suffix),
            "netdev=net0".to_string(),
        ];
    if platform_config.qemu.net == "y" {
        qemu_args_final.extend(qemu_args_net);
    }
    // net_dev
    if platform_config.qemu.net_dev == "user" {
        qemu_args_final.push("-netdev".to_string());
        qemu_args_final.push("user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555".to_string());
    } else if platform_config.qemu.net_dev == "tap" {
        qemu_args_final.push("-netdev".to_string());
        qemu_args_final.push("tap,id=net0,ifname=tap0,script=no,downscript=no".to_string());
    } else {
        log(LogLevel::Error, "NET_DEV must be one of 'user' or 'tap'"); 
        std::process::exit(1);
    }
    // net_dump
    if platform_config.qemu.net_dump == "y" {
        qemu_args_final.push("-object".to_string());
        qemu_args_final.push("filter-dump,id=dump0,netdev=net0,file=netdump.pcap".to_string());
    }
    // graphic
    let qemu_args_graphic = vec![
            "-device".to_string(),
            format!("virtio-gpu-{}", vdev_suffix),
            "-vga".to_string(),
            "none".to_string(),
            "-serial".to_string(),
            "mon:stdio".to_string(),
        ];
    if platform_config.qemu.graphic == "y" {
        qemu_args_final.extend(qemu_args_graphic);
    } else {
        qemu_args_final.push("-nographic".to_string());
    }
    // qemu_log
    if platform_config.qemu.qemu_log == "y" {
        qemu_args_final.push("-D".to_string());
        qemu_args_final.push("qemu.log".to_string());
        qemu_args_final.push("-d".to_string());
        qemu_args_final.push("in_asm,int,mmu,pcall,cpu_reset,guest_errors".to_string());
    }
    qemu_args_final
}
    