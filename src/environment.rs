use crate::utils;
use crate::utils::PlatformConfig;

// Configure environment variables according to the toml file
pub fn config_env(platform_cfg: &PlatformConfig) {
    let platform_name = platform_cfg.name;
    let arch = platform_cfg.arch;
    let smp = platform_cfg.smp;
    let mode = platform_cfg.mode;
    let log = platform_cfg.log;
    let target = match &arch[..] {
        "x86_64" => "x86_64-unknown-none".to_string(),
        "riscv64" => "riscv64gc-unknown-none-elf".to_string(),
        "aarch64" => "aarch64-unknown-none-softfloat".to_string(),
        _ => {
            utils::log(utils::LogLevel::Error, "\"ARCH\" must be one of \"x86_64\", \"riscv64\", or \"aarch64\"");
            std::process::exit(1);
        }
    };

    // Configure environment variables
    std::env::set_var("AX_ARCH", arch);
    std::env::set_var("AX_PLATFORM", platform_name);
    std::env::set_var("AX_SMP", smp);
    std::env::set_var("AX_MODE", mode);
    std::env::set_var("AX_LOG", log);
    std::env::set_var("AX_TARGET", target);
}
