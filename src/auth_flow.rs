use std::{future::Future, time::Duration};

use crate::{dbg, if_logging, utils::await_with_err_log};
use thirtyfour::{By, WebDriver, error::WebDriverError};
use tokio::time::sleep;

use crate::ms_handlers::*;
/// We manage the authentication flow as a state machine for the differnt stages that the user may
/// need to be prompted to do things / preform actions,
///

pub type Error = WebDriverError;

#[derive(Debug, PartialEq, Eq, Clone, Ord, PartialOrd, Hash)]
pub enum AuthState {
    Init,
    CredsPrompt {
        username: Option<String>,
        password: Option<String>,
    },
    AuthSpooling,
    ApproveAppNotif(u64),
    PhoneOTP(Option<String>),
    Failure,
    FailureUserPassword,
    Authenticated,
}

impl AuthState {
    pub fn exit_state(&self) -> bool {
        *self == Self::Failure || *self == Self::Authenticated || *self == Self::FailureUserPassword
    }
}

pub async fn step_auth_sm<'a>(driver: &WebDriver, state: AuthState) -> Result<AuthState, Error> {
    use AuthState::*;

    match state {
        Init => handler_init(driver).await,
        CredsPrompt { username, password } => {
            handler_creds_prompt(driver, username, password).await
        }
        ApproveAppNotif(code_for_phone) => {
            retry_ms_handler(|| handler_phone_notification(driver, code_for_phone)).await
        }
        PhoneOTP(otp) => retry_ms_handler(|| handler_phone_otp(driver, otp.clone())).await,
        AuthSpooling => retry_ms_handler(|| handler_auth_spooling(driver)).await,
        Failure | FailureUserPassword | Authenticated => Ok(state), // shouldn't be called with a finished state but if we are
    }
}

async fn retry_ms_handler<F, Fut>(mut handler: F) -> Result<AuthState, Error>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<AuthState, Error>>,
{
    const MAX_ATTEMPTS: u8 = 3;

    for attempt in 1..=MAX_ATTEMPTS {
        match handler().await {
            Ok(state) => return Ok(state),
            Err(_) if attempt == MAX_ATTEMPTS => return Ok(AuthState::Failure),
            Err(_) => {
                sleep(Duration::from_millis(250)).await;
                continue;
            }
        }
    }

    Ok(AuthState::Failure)
}

async fn handler_init(driver: &WebDriver) -> Result<AuthState, Error> {
    dbg::log!("[init] Start init handler");

    if_logging!(println!("Trying to get https://exampapers.ed.ac.uk/ ..."));
    await_with_err_log!(
        driver.goto("https://exampapers.ed.ac.uk/"),
        "Couldn't fetch https://exampapers.ed.ac.uk/",
    );
    // we expect to be on exampapers.ed.ac.uk meaning were already authed OR https://edadfed.ed.ac.uk/adfs/ls/

    let url = await_with_err_log!(
        driver.current_url(),
        "Couldn't get the current URL after initialization",
    );

    let domain = url.domain().unwrap_or("");
    let path = url.path();
    dbg::log!("[init] waiting on url.domain={} url.path={}", domain, path);

    if domain == "exampapers.ed.ac.uk" {
        return Ok(AuthState::Authenticated);
    }

    if domain == "edadfed.ed.ac.uk" || path == "/adfs/ls/" {
        Ok(AuthState::CredsPrompt {
            username: None,
            password: None,
        })
    } else {
        if_logging!(eprintln!(
            "ERROR! ended up on unknown page https://{}{}",
            domain, path
        ));
        dbg::log!("[init] url={}", url);
        Ok(AuthState::Failure)
    }
}

async fn handler_creds_prompt(
    driver: &WebDriver,
    username: Option<String>,
    password: Option<String>,
) -> Result<AuthState, Error> {
    dbg::log!("[creds_prmt] Start input creds handler");

    if username.is_none() || password.is_none() {
        dbg::log!("[creds_prmt] Didnt get username or password");
        return Ok(AuthState::CredsPrompt {
            username: None,
            password: None,
        });
    }
    let (username, password) = (username.unwrap(), password.unwrap());

    await_with_err_log!(
        driver.find(By::Id("userNameInput")),
        "Couldn't find username input field",
    )
    .send_keys(username)
    .await?;
    dbg::log!("[creds_prmt] inputed username");

    await_with_err_log!(
        driver.find(By::Id("passwordInput")),
        "Couldn't find password input field",
    )
    .send_keys(password)
    .await?;
    dbg::log!("[creds_prmt] inputed password");

    await_with_err_log!(
        driver.find(By::Id("submitButton")),
        "Couldn't find the submit button",
    )
    .click()
    .await?;
    dbg::log!("[creds_prmt] clicked submit");

    let url = await_with_err_log!(
        driver.current_url(),
        "Couldn't get the current URL after submitting credentials",
    );
    dbg::log!("[creds_prmt] submission took us too {}", url);
    if url.as_str() != "https://login.microsoftonline.com/login.srf" {
        if_logging!(eprintln!(
            "Unable to authenticate with provided username and password\nIf you're sure you're using the correct email and password open an issue on the GH as the API may have changed"
        ));
        return Ok(AuthState::FailureUserPassword);
    }
    sleep(Duration::from_millis(250)).await;

    Ok(AuthState::AuthSpooling)
}
