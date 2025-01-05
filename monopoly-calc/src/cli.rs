use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Print debugging messages
    #[arg(short, long)]
    pub debug: bool,

    /// Desired decimal places accuracy
    #[arg(short='a', long, default_value_t = 8, value_parser = clap::value_parser!(u8).range(1..=15))]
    pub dp: u8,
}
