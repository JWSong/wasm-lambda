use clap::{Parser, Subcommand, arg, command};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "wasm-lambda")]
#[command(about = "Simple WASM function runner triggered by events")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Deploy a new function with the given name and WASM file.
    /// * name: The name of the function
    /// * file: The path to the WASM file
    /// * trigger: The triggers of the function    
    Deploy {
        #[arg(short, long)]
        name: String,

        #[arg(short, long)]
        file: PathBuf,

        #[arg(short, long)]
        trigger: Vec<String>,
    },

    /// List all deployed functions
    List,

    /// Invoke a function directly
    /// * id: The id of the function
    /// * subject: The subject of the function
    /// * data: The data of the function
    Invoke {
        #[arg(short, long)]
        id: String,

        #[arg(short, long)]
        subject: String,

        #[arg(short, long)]
        data: String,
    },

    /// Start the server
    Start,
}
