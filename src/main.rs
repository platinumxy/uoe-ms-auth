use std::io::{self, Write};

use thirtyfour::prelude::*;
mod auth_flow;
mod dbg;
use auth_flow::AuthState;

#[tokio::main]
async fn main() {
    if_logging!(print!(
        "Starting web-driver may take a while on first run..."
    ));
    let driver = create_driver().await.unwrap();

    if_logging!(println!("Done"));

    let mut state = AuthState::Init;


    while !state.exit_state() {
        // See if we need to prompt the user for the state machine 
        match &state {  
            AuthState::CredsPrompt {
                username: _,
                password: _,
            } => {
                let (user, pass) = get_creds();
                state = AuthState::CredsPrompt {
                    username: Some(user),
                    password: Some(pass),
                }
            }
            _ => (), 
        };


        state = auth_flow::step_auth_sm(&driver, state).await.unwrap();
    }

    if_logging!(println!("Exit state {:?}", state));
}

macro_rules! if_logging {
    ($($code:tt)*) => {
        #[cfg(not(feature = "no-logging"))]
        {
            $($code)*
        }
    };
}
pub(crate) use if_logging;

fn get_creds() -> (String, String) {
    let username = loop {
        print!("Please enter your student email (i.e. s1234567@ed.ac.uk): ");

        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let trimmed = input.trim().to_string();

        if trimmed.ends_with("@ed.ac.uk") {
            break trimmed;
        }
        println!("Invalid email. Please enter an email ending with @ed.ac.uk");
    };

    let password = rpassword::prompt_password("Your password: ").unwrap();
    (username, password)
}

async fn create_driver() -> Result<WebDriver, WebDriverError> {
    let mut caps = DesiredCapabilities::chrome(); //TODO: See if we can auth find a browser to skip
    //download

    #[cfg(not(feature = "show-browser"))]
    if let Err(err) = caps.set_headless() {
        eprintln!("Could not tell driver to be headless: {}", err);
        return Err(err);
    }

    let driver = WebDriver::managed(caps).await;
    if let Err(err) = &driver {
        eprintln!("Couldn't create managed web driver: {}", err);
    }
    driver
}
