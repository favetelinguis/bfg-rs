mod app;
mod components;
mod rest;
mod stream;

use clap::Parser;
use color_eyre::{
    eyre::{self, Context, Ok},
    Section,
};
use std::path::PathBuf;

use app::model::Model;
use directories::ProjectDirs;
use tuirealm::{AttrValue, Attribute, PollStrategy, Update};

use crate::stream::{AuthenticationMessage, LinesCodec};

// What messages the app can handle, must have `PartialEq`
#[derive(Debug, PartialEq)]
pub enum Msg {
    AppClose,
    Clock,
    DigitCounterChanged(isize),
    DigitCounterBlur,
    LetterCounterChanged(isize),
    LetterCounterBlur,
    None,
}

// Let's define the component ids for our application
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    Ladder,
    Markets,
    Status,
    Phantom,
}

#[derive(Parser, Debug)]
#[command(version = "1", about = "a betfair trading tui")]
struct Args {
    #[arg(short, long, default_value_t = 1000)]
    app_tick_rate: u64,
}

fn get_data_dir() -> eyre::Result<PathBuf> {
    let directory = if let Some(proj_dirs) = ProjectDirs::from("com", "flimmer", "bfg") {
        proj_dirs.data_local_dir().to_path_buf()
    } else {
        return Err(eyre::eyre!("Unable to find data directory for bfg"));
    };
    Ok(directory)
}

fn get_config_dir() -> eyre::Result<PathBuf> {
    let directory = if let Some(proj_dirs) = ProjectDirs::from("com", "flimmer", "bfg") {
        proj_dirs.config_local_dir().to_path_buf()
    } else {
        return Err(eyre::eyre!("Unable to find config directory for bfg"));
    };
    Ok(directory)
}

#[derive(Debug)]
pub struct ConnectionConfig {
    pub app_key: String,
    pub password: String,
    pub username: String,
}

impl ConnectionConfig {
    pub fn new() -> eyre::Result<Self> {
        let username = std::env::var("BFG_USERNAME").wrap_err("BFG_USERNAME not set in env")?;
        let password = std::env::var("BFG_PASSWORD").wrap_err("BFG_PASSWORD not set in env")?;
        let app_key = std::env::var("BFG_APP_KEY").wrap_err("BFG_APP_KEY not set in env")?;

        Ok(ConnectionConfig {
            username,
            password,
            app_key,
        })
    }
}

fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let conf = ConnectionConfig::new()?;
    let login_res = rest::login(
        &conf.app_key,
        &conf.username,
        &conf.password,
        get_config_dir()?,
    )?;
    let mut s = LinesCodec::new()?;
    let auth_msg = AuthenticationMessage::new(&conf.app_key, &login_res.session_token.unwrap());

    let res = s.send_message(auth_msg)?;
    let res = s.read_message()?;
    println!("{:?}", res);
    let res = s.read_message()?;
    println!("{:?}", res);
    let res = s.read_message()?;
    println!("{:?}", res);

    Ok(())

    // // Setup model
    // let mut model = Model::default();

    // // Setup terminal
    // let _ = model.terminal.enter_alternate_screen();
    // let _ = model.terminal.enable_raw_mode();

    // // TODO main loop
    // while !model.quit {
    //     match model.app.tick(PollStrategy::Once) {
    //         Err(err) => {
    //             assert!(model
    //                 .app
    //                 .attr(
    //                     &Id::Status,
    //                     Attribute::Title,
    //                     AttrValue::String(format!("Application error: {}", err)),
    //                 )
    //                 .is_ok());
    //         }
    //         // Handle the Msg sent in app by calling update
    //         Ok(messages) if messages.len() > 0 => {
    //             model.redraw = true;
    //             for msg in messages.into_iter() {
    //                 let mut msg = Some(msg);
    //                 while msg.is_some() {
    //                     // TODO I call update on model but how is update on components called?
    //                     msg = model.update(msg);
    //                 }
    //             }
    //         }
    //         _ => {}
    //     }
    //     // Redraw
    //     if model.redraw {
    //         model.view(); // TODO call view on app how to connect components?
    //         model.redraw = false;
    //     }
    // }

    // // TODO maybe i need som handle for panics with hooks, is in ratatue manual
    // // Also might want to add clap for arguments?
    // // Restore terminal
    // let _ = model.terminal.leave_alternate_screen();
    // let _ = model.terminal.disable_raw_mode();
    // let _ = model.terminal.clear_screen();
}
