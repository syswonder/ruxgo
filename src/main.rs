use rukoskit::utils;
use rukoskit::builder;
fn main() {
    utils::log(utils::LogLevel::Info, "Hello, Rukoskit!");
    let (build_config, targets) = utils::parse_config("./config_linux.toml");
    for target in targets{
        let _target=builder::Target::new(&build_config,&target);
    }
}