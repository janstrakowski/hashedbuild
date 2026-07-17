use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "hashedbuild")]
#[command(about = "The command line interface of Hashedbuild build system.", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Eval {
        #[arg(short, long)]
        argument: Option<String>,

        #[arg(short, long)]
        file: String,

        #[arg(short, long)]
        source: String,
    },
}
