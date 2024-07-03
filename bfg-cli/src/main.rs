use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all markets
    ListMarkets,
    /// Create a new order
    Create {
        /// The market to use
        market: String,
    },
    /// Login to betfair
    Login,
    /// Logout from betfair
    Logout,
    /// Left this here as a more advanced example
    Todo {
        /// The id
        id: i64,
        /// The body
        body: String,
        /// Mark as complete
        #[arg(short, long)]
        completed: bool,
    },
}

#[derive(Parser)]
struct Cli {
    /// The pattern to look for
    // globalarg: String,
    #[command(subcommand)]
    command: Commands,
}

fn main() -> Result<()> {
    ctrlc::set_handler(move || log::info!("Ctrl+C pressed")).expect("Error setting Ctrl-C handler");
    let cli = Cli::parse();
    env_logger::init();

    match cli.command {
        Commands::Login => {
            let config = bfg_betfair::env::ConnectionConfig::new()?;
            let resp = bfg_betfair::rest::login(
                &config.url,
                &config.app_key,
                &config.username,
                &config.password,
            )?;
            log::debug!("running login {:?}", resp);
        }
        _ => {
            unimplemented!();
        }
    }

    Ok(())
}
