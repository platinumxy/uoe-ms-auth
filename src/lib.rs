use std::io::{self, Write};

pub mod auth_flow;
pub mod bindings;
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
    let (username, password) = get_creds();
    run_with_credentials(
        username,
        password,
        || Ok(get_otp()),
        |number| {
            let _ = number;
            if_logging!(println!("Approve the signin request for code: {}", number));
            Ok(())
        },
    )
        .await
        .map_err(|error| eprintln!("Authentication failed: {error}\x1b[0K"))
        .ok()
}

/// Run authentication with host-provided credentials and OTP input.
pub async fn run_with_credentials<F>(
    username: String,
    password: String,
    mut otp_provider: F,
    mut approval_notifier: impl FnMut(u64) -> Result<(), String>,
) -> Result<String, String>
where
    F: FnMut() -> Result<String, String>,
{
    if !username.ends_with("@ed.ac.uk") {
        return Err("username must end with @ed.ac.uk".to_string());
    }
    if password.is_empty() {
        return Err("password must not be empty".to_string());
    }

    utils::if_logging!(print!(
        "Starting web-driver may take a while on first run..."
    ));
    let driver = utils::create_driver()
        .await
        .map_err(|error| format!("could not create web driver: {error}"))?;
    utils::if_logging!(println!("Done"));

    let mut state = AuthState::Init;
    while !state.exit_state() {
        match &state {
            AuthState::CredsPrompt { .. } => {
                state = AuthState::CredsPrompt {
                    username: Some(username.clone()),
                    password: Some(password.clone()),
                };
            }
            AuthState::ApproveAppNotif(number) => {
                approval_notifier(*number)?;
            }
            AuthState::PhoneOTP(_) => {
                state = AuthState::PhoneOTP(Some(otp_provider()?));
            }
            _ => (),
        };

        state = auth_flow::step_auth_sm(&driver, state)
            .await
            .map_err(|error| format!("authentication failed: {error}"))?;
    }

    if state != AuthState::Authenticated {
        return Err(format!("authentication failed: {state:?}"));
    }
    if_logging!(println!("Authenticated successfully!\x1b[0K"));
    Ok(utils::cookies_from(&driver,     vec![
        "edadfed.ed.ac.uk",
        "exampapers.ed.ac.uk",
        "idp.ed.ac.uk",
        "login.live.com",
        "login.microsoft.com",
        "login.microsoftonline.com",
    ]).await)
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
