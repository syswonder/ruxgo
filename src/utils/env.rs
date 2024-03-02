//! Environment Configuration

use crate::parser::{OSConfig, PlatformConfig, QemuConfig};

// This function is used to configure environment variables
pub fn config_env(os_config: &OSConfig,) {
    if os_config != &OSConfig::default() && os_config.platform != PlatformConfig::default() {
        std::env::set_var("RUX_ARCH", &os_config.platform.arch);
        std::env::set_var("RUX_PLATFORM", &os_config.platform.name);
        std::env::set_var("RUX_SMP", &os_config.platform.smp);
        std::env::set_var("RUX_MODE", &os_config.platform.mode);
        std::env::set_var("RUX_LOG", &os_config.platform.log);
        std::env::set_var("RUX_TARGET", &os_config.platform.target);
        if os_config.platform.qemu != QemuConfig::default() {
            // ip and gw is for QEMU user netdev
            std::env::set_var("RUX_IP", &os_config.platform.qemu.ip);
            std::env::set_var("RUX_GW", &os_config.platform.qemu.gw);
            // v9p option
            if os_config.platform.qemu.v9p == "y" {
                std::env::set_var("RUX_9P_ADDR", "127.0.0.1:564");
                std::env::set_var("RUX_ANAME_9P", "./");
                std::env::set_var("RUX_PROTOCOL_9P", "9P2000.L");
            }
        }
        // musl
        if os_config.ulib == "ruxmusl" {
            std::env::set_var("RUX_MUSL", "y");
        }
    }
}
