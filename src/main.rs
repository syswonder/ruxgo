use rukoskit::{utils, commands};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use rukoskit::global_cfg::GlobalConfig;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Build your project
    #[arg(short, long)]
    build: bool,
    /// Clean the obj and bin intermediates
    #[arg(short, long)]
    clean: bool,
    /// Run the executable
    #[arg(short, long)]
    run: bool,
    /// Initialize a new project. See `init --help` for more info
    #[command(subcommand)]
    commands: Option<Commands>,
    /// Arguments to pass to the executable when running
    #[arg(long, num_args(1..))]
    bin_args: Option<Vec<String>>,
    /// Generate compile_commands.json
    #[arg(long)]
    gen_cc: bool,
    /// Generate .vscode/c_cpp_properties.json
    #[arg(long)]
    gen_vsc: bool,
    /// Clean packages
    #[arg(long)]
    clean_packages: bool,
    /// Update packages
    #[arg(long)]
    update_packages: bool,
    /// Restore packages
    #[arg(long)]
    restore_packages: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a new project
    /// Defaults to C++ if no language is specified
    Init {
        /// Name of the project
        name: String,
        #[clap(long, action)]
        /// Initialize a C project
        c: bool,
        #[clap(long, action)]
        /// Initialize a C++ project
        cpp: bool,
    },
    /// Configuration settings
    Config {
        /// Parameter to set currently supported parameters:
        ///     - `default_compiler`: Sets the default compiler to use
        ///     - `default_language`: Sets the default language to use
        ///     - `license`: Sets the license to use. Give the path to the license file
        #[clap(verbatim_doc_comment)]
        parameter: String,
        /// Value to set the parameter to currently supported values:
        ///     - `compiler`: `gcc`, `clang` Uses g++ or clang++ respectively
        ///     - `language`: `c`, `cpp`
        ///     - `license`: `path/to/license/file`
        #[clap(verbatim_doc_comment)]
        value: String,
    },
}

fn main() {
    // Add global_config
    let project_dirs = ProjectDirs::from("com", "RukosApps", "rukos-c").unwrap();
    let config_dir = project_dirs.config_dir();
    if !config_dir.exists() {
        std::fs::create_dir_all(config_dir).unwrap();
    }
    let config = config_dir.join("config.toml");
    if !config.exists() {
        std::fs::write(
            &config,
            r#"
default_compiler = "gcc"
default_language = "cpp"
license = "NONE"
"#,
        )
        .unwrap();
    }
    let global_config = GlobalConfig::from_file(&config);

    // Parse args
    let args = Args::parse();
    if args.commands.is_some() {
        match args.commands {
            Some(Commands::Init { name, c, cpp }) => {
                if c && cpp {
                    utils::log(
                        utils::LogLevel::Error,
                        "Only one of --c or --cpp can be specified",
                    );
                    std::process::exit(1);
                }
                if !c && !cpp {
                    commands::init_project(&name, None, &global_config);
                }
                if c {
                    commands::init_project(&name, Some(true), &global_config);
                } else {
                    commands::init_project(&name, Some(false), &global_config);
                }
            }
            Some(Commands::Config { parameter, value }) => {
                let parameter = parameter.as_str();
                let value = value.as_str();
                GlobalConfig::set_defaults(&config, parameter, value);
                utils::log(
                    utils::LogLevel::Log,
                    format!("Setting {} to {}", parameter, value).as_str(),
                );
                std::process::exit(0);
            }
            None => {
                utils::log(utils::LogLevel::Error, "Rust is broken");
                std::process::exit(1);
            }
        }
    }

    let mut gen_cc = false;
    if args.gen_cc {
        gen_cc = true;
        commands::pre_gen_cc();
    }

    let mut gen_vsc = false;
    if args.gen_vsc {
        gen_vsc = true;
        commands::pre_gen_vsc();
    }

    let (build_config, os_config, targets, packages) = commands::parse_config();

    if args.clean_packages {
        commands::clean_packages(&packages);
        std::process::exit(0);
    }

    if args.update_packages {
        commands::update_packages(&packages);
        std::process::exit(0);
    }

    if args.restore_packages {
        commands::restore_packages(&packages);
        std::process::exit(0);
    }

    if args.clean {
        utils::log(utils::LogLevel::Log, "Cleaning...");
        commands::clean(&targets, &os_config, &packages);
    }

    if args.build {
        utils::log(utils::LogLevel::Log, "Building...");
        commands::build(&build_config, &targets, &os_config, gen_cc, gen_vsc, &packages);
    }

    if args.run {
        let bin_args: Option<Vec<&str>> = args
            .bin_args
            .as_ref()
            .map(|x| x.iter().map(|x| x.as_str()).collect());

        utils::log(utils::LogLevel::Log, "Running...");
        let exe_target = targets.iter().find(|x| x.typ == "exe").unwrap();
        commands::run(bin_args, &build_config, &os_config, exe_target, &targets, &packages);
    }
}

