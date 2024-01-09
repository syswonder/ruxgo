use ruxgo::utils::OSConfig;
use ruxgo::{utils, commands};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use ruxgo::global_cfg::GlobalConfig;
use ruxgo::packages;
use dialoguer::MultiSelect;
use std::env;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CLIArgs {
    /// Build your project
    #[arg(short, long)]
    build: bool,
    /// Clean the obj and bin intermediates
    #[arg(short, long)]
    clean: bool,
    /// Choose which parts to delete
    #[arg(conflicts_with("clean"))]
    choices: Vec<String>,
    /// Run the executable
    #[arg(short, long)]
    run: bool,
    /// Initialize a new project. See `init --help` for more info
    #[command(subcommand)]
    commands: Option<Commands>,
    /// Path argument to pass to switch to the specified directory
    #[arg(long, num_args(1))]
    path: Option<PathBuf>,
    /// Arguments to pass to the executable when running
    #[arg(long, num_args(1..), require_equals(true), value_delimiter(','))]
    bin_args: Option<Vec<String>>,
    /// Generate compile_commands.json
    #[arg(long)]
    gen_cc: bool,
    /// Generate .vscode/c_cpp_properties.json
    #[arg(long)]
    gen_vsc: bool,
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
    /// Package management
    #[clap(name = "pkg", arg_required_else_help = true)]
    Pkg {
        /// List available packages in the remote repository
        #[arg(short, long)]
        list: bool,
        /// Pull a specific package from the remote repository
        #[clap(short, long, value_name = "PKG_NAME")]
        pull: Option<String>,
        /// Run a specific app-bin
        #[clap(short, long, value_name = "APP_BIN")]
        run: Option<String>,
        /// Update a specific package
        #[clap(short, long, value_name = "PKG_NAME")]
        update: Option<String>,
        /// Clean a specific package
        #[clap(short, long, value_name = "PKG_NAME")]
        clean: Option<String>,
        /// Clean all packages
        #[arg(long)]
        clean_all: bool,
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

#[tokio::main]
async fn main() {
    // Add global config
    let project_dirs = ProjectDirs::from("com", "RuxosApps", "ruxos-c").unwrap();
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
    let args = CLIArgs::parse();

    if let Some(ref path_buf) = args.path {
        if let Err(e) = env::set_current_dir(&path_buf) {
            eprintln!("Error path: {}", e);
        }
    }

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
            Some(Commands::Pkg { list, pull, run, update, clean, clean_all }) => {
                if list {
                    packages::list_packages().await.expect("Failed to list packages");
                }
                if let Some(pkg_name) = pull {
                    packages::pull_packages(&pkg_name).await.expect("Failed to pull package");
                }
                if let Some(app_name) = run {
                    packages::run_app(&app_name).expect("Failed to run app-bin");
                }
                if let Some(pkg_name) = update {
                    packages::update_package(&pkg_name).await.expect("Failed to update package");
                }
                if let Some(pkg_name) = clean {
                    packages::clean_package(&pkg_name).await.expect("Failed to clean package");
                }
                if clean_all {
                    let items = vec!["All", "App-bin", "App-src", "Kernel", "Script", "Cache"];
                    let defaults = vec![false; items.len()];
                    let choices = MultiSelect::new()
                        .with_prompt("What parts do you want to clean?")
                        .items(&items)
                        .defaults(&defaults)
                        .interact_opt()
                        .unwrap_or_else(|_| None)
                        .unwrap_or_else(|| Vec::new())
                        .iter()
                        .map(|&index| String::from(items[index]))
                        .collect();
                    utils::log(utils::LogLevel::Log, "Cleaning packages...");
                    packages::clean_all_packages(choices).expect("Failed to clean choice packages");
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

    // If clean flag is provided, prompt user for choices
    if args.clean {
        let (_, os_config, targets, packages) = commands::parse_config();
        let mut items = vec!["All", "App_bins", "Obj"];
        if os_config != OSConfig::default() {
            items.push("OS");
            if !os_config.ulib.is_empty() {
                items.push("Ulib");
            }
        }
        if !packages.is_empty() {
            items.push("Packages");
        }
        let defaults = vec![false; items.len()];
        let choices = MultiSelect::new()
            .with_prompt("What parts do you want to clean?")
            .items(&items)
            .defaults(&defaults)
            .interact_opt()
            .unwrap_or_else(|_| None)
            .unwrap_or_else(|| Vec::new())
            .iter()
            .map(|&index| String::from(items[index]))
            .collect();

        utils::log(utils::LogLevel::Log, "Cleaning...");
        commands::clean(&targets, &os_config, &packages, choices);
    }

    if args.build {
        let (build_config, os_config, targets, packages) = commands::parse_config();
        utils::log(utils::LogLevel::Log, "Building...");
        commands::build(&build_config, &targets, &os_config, gen_cc, gen_vsc, &packages);
    }

    if args.run {
        let (build_config, os_config, targets, packages) = commands::parse_config();
        let bin_args: Option<Vec<&str>> = args.bin_args
            .as_ref()
            .map(|x| x.iter().map(|x| x.as_str()).collect());

        utils::log(utils::LogLevel::Log, "Running...");
        let exe_target = targets.iter().find(|x| x.typ == "exe").unwrap();
        commands::run(bin_args, &build_config, &os_config, exe_target, &targets, &packages);
    }
}

