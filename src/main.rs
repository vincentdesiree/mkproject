use clap::Parser;
use mkproject::cli::Cli;
use mkproject::config::Config;
use mkproject::generator::generate_project;
use mkproject::project::ResolvedProject;

fn main() {
    let cli = Cli::parse();
    let config = Config::load();
    let resolved = ResolvedProject::resolve(cli, config);

    generate_project(&resolved);
}
