use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "yakitsuke", about = "Safe ROM flasher")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Flash a ROM image to a device
    Flash {
        /// Path to the image file
        image: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Flash { image } => {
            println!("yakitsuke: flashing {image}");
        }
    }
}
