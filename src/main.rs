use rukoskit::{utils, commands};
use clap::{Parser, Subcommand};

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
    init: Option<Commands>,
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
}

fn main() {
    let args = Args::parse();

    if args.init.is_some() {
        match args.init {
            Some(Commands::Init { name, c, cpp }) => {
                if c && cpp {
                    utils::log(
                        utils::LogLevel::Error,
                        "Only one of --c or --cpp can be specified",
                    );
                    std::process::exit(1);
                }
                if !c && !cpp {
                    utils::log(
                        utils::LogLevel::Warn,
                        "No language specified. Defaulting to C++",
                    );
                    commands::init_project(&name, true);
                    std::process::exit(0);
                }
                if c {
                    commands::init_project(&name, true);
                } else {
                    commands::init_project(&name, false);
                }
            }
            None => {
                utils::log(utils::LogLevel::Error, "No init command specified");
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
        commands::clean(&targets);
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

