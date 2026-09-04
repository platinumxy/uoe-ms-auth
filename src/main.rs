use std::io::{self, Write};

pub mod auth_flow;
pub mod disable_webauth;
pub mod ms_handlers;

pub mod dbg;
pub mod utils;

use auth_flow::AuthState;
use utils::if_logging;

#[tokio::main]
async fn main() {
    if_logging!(print!(
        "Starting web-driver may take a while on first run..."
    ));
    let driver = match utils::create_driver().await {
        Ok(driver) => driver,
        Err(error) => {
            eprintln!("Could not create web driver: {error}");
            return;
        }
    };

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
            AuthState::ApproveAppNotif(number) => {
                print!("Please approve the signin request for code: {}\r", number); // \r quick
                // and dirty
                // hack to
                // only show
                // one msg
                io::stdout().flush().unwrap();
            }
            AuthState::PhoneOTP(_) => state = AuthState::PhoneOTP(Some(get_otp())),
            _ => (),
        };

        state = match auth_flow::step_auth_sm(&driver, state).await {
            Ok(next_state) => next_state,
            Err(error) => {
                eprintln!("Authentication failed: {error}");
                return;
            }
        };
    }

    if_logging!(println!(
        "Exit state {:?}                                   ",
        state
    ));
    println!(
        "Cookies: {}",
        utils::cookies_from(
            &driver,
            vec![
                "edadfed.ed.ac.uk",
                "exampapers.ed.ac.uk",
                "idp.ed.ac.uk",
                "login.live.com",
                "login.microsoft.com",
                "login.microsoftonline.com"
            ]
        )
        .await
    )
}

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

fn get_otp() -> String {
    loop {
        print!("Please enter an OTP from your microsoft authenticator app: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let trimmed = input.trim().to_string();
        if trimmed.parse::<f64>().is_ok() {
            break trimmed;
        }
    }
}
