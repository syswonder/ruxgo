use rukoskit::utils;
use rukoskit::builder;
fn main() {
    utils::log(utils::LogLevel::Info, "Hello, world!");
    let (build_config, targets) = utils::parse_config("./config_linux.toml");
    println!("{:?}", build_config);
    println!("{:?}", targets);
    for target in targets{
        let _target=builder::Target::new(&build_config,&target);
    }
}