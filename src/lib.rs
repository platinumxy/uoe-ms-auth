use std::io::{self, Write};

pub mod auth_flow;
pub mod dbg;
pub mod disable_webauth;
pub mod ms_handlers;
pub mod utils;

pub(crate) use utils::if_logging;

pub use auth_flow::{AuthState, Error, step_auth_sm};
pub use utils::{cookies_from, create_driver, current_page_cookies};

/// Run the interactive authentication flow using a managed WebDriver.
///
/// Returns `Some` with the authenticated cookies, or `None` if authentication
/// fails.
pub async fn run() -> Option<String> {
    utils::if_logging!(print!(
        "Starting web-driver may take a while on first run..."
    ));
    let driver = match utils::create_driver().await {
        Ok(driver) => driver,
        Err(error) => {
            eprintln!("Could not create web driver: {error}");
            return None;
        }
    };

    utils::if_logging!(println!("Done"));

    let mut state = AuthState::Init;

    while !state.exit_state() {
        match &state {
            AuthState::CredsPrompt { .. } => {
                let (user, pass) = get_creds();
                state = AuthState::CredsPrompt {
                    username: Some(user),
                    password: Some(pass),
                };
            }
            AuthState::ApproveAppNotif(number) => {
                print!("Please approve the signin request for code: {number}\r");
                io::stdout().flush().unwrap();
            }
            AuthState::PhoneOTP(_) => state = AuthState::PhoneOTP(Some(get_otp())),
            _ => (),
        };

        state = match auth_flow::step_auth_sm(&driver, state).await {
            Ok(next_state) => next_state,
            Err(error) => {
                eprintln!("Authentication failed: {error}\x1b[0K");
                return None;
            }
        };
    }

    if state != AuthState::Authenticated {
        eprintln!("Authentication failed: {state:?}\x1b[0K");
        return None;
    }
    if_logging!(println!("Authenticated successfully!\x1b[0K"));
    Some(
        utils::cookies_from(
            &driver,
            vec![
                "edadfed.ed.ac.uk",
                "exampapers.ed.ac.uk",
                "idp.ed.ac.uk",
                "login.live.com",
                "login.microsoft.com",
                "login.microsoftonline.com",
            ],
        )
        .await,
    )
}

pub fn get_creds() -> (String, String) {
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

pub fn get_otp() -> String {
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
