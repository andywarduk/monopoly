use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Roll to get out of jail
    #[arg(short, long)]
    pub wait: bool,
}
