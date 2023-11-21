//use crate::utils;

pub fn config_env() {
    //? Configure environment variables according to the toml file
    let arch = "x86_64".to_string();
    let platform_name = "x86_64-qemu-q35".to_string();
    let smp = "1".to_string();
    let mode = "release".to_string();
    let log = "warn".to_string();
    let target = "x86_64-unknown-none".to_string();

    //utils::log(utils::LogLevel::Debug, "Configure environment variables");
    std::env::set_var("AX_ARCH", arch);
    std::env::set_var("AX_PLATFORM", platform_name);
    std::env::set_var("AX_SMP", smp);
    std::env::set_var("AX_MODE", mode);
    std::env::set_var("AX_LOG", log);
    std::env::set_var("AX_TARGET", target);
}
