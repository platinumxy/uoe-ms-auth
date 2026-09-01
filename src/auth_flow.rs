use crate::{dbg, if_logging};
use thirtyfour::{By, WebDriver, error::WebDriverError};

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
    ApproveSigninCode(u64),
    AwaitingPhoneCode(Option<u64>),
    Failure,
    FailureUserPassword,
    Authenticated,
}

impl AuthState {
    pub fn exit_state(&self) -> bool {
        *self == Self::Failure || *self == Self::Authenticated || *self == Self::FailureUserPassword
    }
}

pub async fn step_auth_sm<'a>(
    driver: &WebDriver,
    state: AuthState,
) -> Result<AuthState, Error> {
    use AuthState::*;

    match state {
        Init => handler_init(driver).await,
        CredsPrompt { username, password } => {
            handler_creds_prompt(driver, username, password).await
        }
        ApproveSigninCode(code_for_phone) => {
            handler_approve_sigin_code(driver, code_for_phone).await
        }
        AwaitingPhoneCode(code_from_phone) => {
            handler_awaiting_phone_code(driver, code_from_phone).await
        }
        AuthSpooling => handler_auth_spooling(driver).await,
        Failure | FailureUserPassword | Authenticated => Ok(state), // shouldn't be called with a finished state but if we are
    }
}

async fn handler_init(driver: &WebDriver) -> Result<AuthState, Error> {
    dbg::log!("[init] Start init handler");

    if_logging!(println!("Trying to get https://exampapers.ed.ac.uk/ ..."));
    driver
        .goto("https://exampapers.ed.ac.uk/")
        .await
        .expect("Could not fetch https://exampapers.ed.ac.uk/");
    // we expect to be on exampapers.ed.ac.uk meaning were already authed OR https://edadfed.ed.ac.uk/adfs/ls/

    let url = driver.current_url().await?;

    let domain = url.domain().unwrap_or("");
    let path = url.path();
    dbg::log!("[init] waiting on url.domain={} url.path={}", domain, path);

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

    driver
        .find(By::Id("userNameInput"))
        .await
        .map_err(|e| {
            if_logging!(eprintln!("Couldn't find username input field"));
            e
        })?
        .send_keys(username)
        .await?;
    dbg::log!("[creds_prmt] inputed username");

    driver
        .find(By::Id("passwordInput"))
        .await
        .map_err(|e| {
            if_logging!(eprintln!("Couldn't find password input field"));
            e
        })?
        .send_keys(password)
        .await?;
    dbg::log!("[creds_prmt] inputed password");

    //driver Appears as defaults to onn and the user cannot see it ?
    //     .find(By::Id("kmsiInput"))
    //     .await
    //     .map_err(|e| {
    //         if_logging!(eprintln!("Couldn't find the keep me signed in button"));
    //         e
    //     })?
    //     .click()
    //     .await?;
    // dbg::log!("[creds_prmt] clicked keep me signed in");

    driver
        .find(By::Id("submitButton"))
        .await
        .map_err(|e| {
            if_logging!(eprintln!("Couldn't find the submit button"));
            e
        })?
        .click()
        .await?;
    dbg::log!("[creds_prmt] clicked submit");

    let url = driver.current_url().await?;
    dbg::log!("[creds_prmt] submission took us too {}", url);
    if url.as_str() != "https://login.microsoftonline.com/login.srf" {
        if_logging!(eprintln!(
            "Unable to authenticate with provided username and password\nIf you're sure you're using the correct email and password open an issue on the GH as the API may have changed"
        ));
        return Ok(AuthState::FailureUserPassword);
    }

    Ok(AuthState::AuthSpooling)
}

async fn handler_auth_spooling(driver: &WebDriver) -> Result<AuthState, Error> {
    dbg::current_url(driver).await;
    Ok(AuthState::AuthSpooling)
}

async fn handler_approve_sigin_code(
    driver: &WebDriver,
    code_for_phone: u64,
) -> Result<AuthState, Error> {
    dbg::log!("Approve this signin code");
    // Poll to see if the browser recognised us having signed in
    Ok(AuthState::Failure)
}

async fn handler_awaiting_phone_code(
    driver: &WebDriver,
    code_from_phone: Option<u64>,
) -> Result<AuthState, Error> {
    dbg::log!("Waiting for code input");
    if code_from_phone.is_none() {
        dbg::log!(
            "Not recived a code, should possibly return a failure state instead? Or we can still leave it to the caller"
        );
        return Ok(AuthState::AwaitingPhoneCode(None));
    }

    Ok(AuthState::Failure)
}
