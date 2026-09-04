use crate::{
    auth_flow::{AuthState, Error},
    dbg, if_logging,
    utils::await_with_err_log,
};
use thirtyfour::{By, WebDriver, error::WebDriverError, extensions::query::ElementQueryable};
use tokio::time::{Duration, sleep};

#[derive(PartialEq, Eq)]
pub enum Ms2faStates {
    TrustWebsite,
    TryingPassKeys,
    CouldntPasskey,
    ChooseVerificationOption,
    PhoneAppNotification,
    PhoneOTP,
    IncorrectCode,
    StaySignedIn,
    Unknown,
}

pub async fn handler_auth_spooling(driver: &WebDriver) -> Result<AuthState, Error> {
    use Ms2faStates::*;
    dbg::log!("[auth_spool] auth spooling");

    match determin_2fa_state(driver).await? {
        TrustWebsite => {
            dbg::log!("[auth_spool][trust_site] trusting ed.ac.uk");
            let button = driver
                .query(By::Id("idSIButton9"))
                .and_clickable()
                .wait(Duration::from_secs(10), Duration::from_millis(100))
                .desc("trust ed.ac.uk button")
                .first()
                .await?;
            sleep(Duration::from_millis(250)).await;
            await_with_err_log!(button.click(), "Couldn't click the trust ed.ac.uk button");
            driver
                .query(By::XPath("//form[contains(@action, '/appverify')]"))
                .wait(Duration::from_secs(5), Duration::from_millis(100))
                .not_exists()
                .await?;
            Ok(AuthState::AuthSpooling)
        }
        ChooseVerificationOption => {
            dbg::log!("[auth_spool][choose_ver_opt] init");
            //todo check we can acc do phone otp
            let option = await_with_err_log!(
                driver.find(By::XPath("//div[@data-value='PhoneAppNotification']")),
                "Couldn't find the phone notification option",
            );
            await_with_err_log!(
                option.click(),
                "Couldn't click the phone notification option",
            );
            dbg::log!("[auth_spool][choose_ver_opt] Clicked PhoneAppOTP");
            driver
                .query(By::Id("idRichContext_DisplaySign"))
                .or(By::Id("idTxtBx_SAOTCC_OTC"))
                .wait(Duration::from_secs(5), Duration::from_millis(100))
                .first()
                .await?;
            Ok(AuthState::AuthSpooling)
        }
        PhoneAppNotification => {
            dbg::log!("[auth_spool][phone notif] reading code ");
            let approval_element = await_with_err_log!(
                driver.find(By::Id("idRichContext_DisplaySign")),
                "Couldn't find the phone approval code",
            );
            let approval_code = await_with_err_log!(
                approval_element.text(),
                "Couldn't read the phone approval code",
            );
            dbg::log!("[auth_spool][phone notif] approval_code={}", approval_code);
            match approval_code.parse::<u64>() {
                Ok(code) => Ok(AuthState::ApproveAppNotif(code)),
                Err(_) => {
                    if_logging!(eprintln!("The phone approval code was not a number"));
                    Ok(AuthState::Failure)
                }
            }
        }
        PhoneOTP => {
            dbg::log!("[auth_spool][phoneOTP] Entering the OTP from a phone");
            Ok(AuthState::PhoneOTP(None)) // we have to wait for the user to give us a code
        }
        StaySignedIn => {
            dbg::log!("[auth_spool][stay_singed_in] telling it to stay signed in");
            let button = await_with_err_log!(
                driver.find(By::Id("idSIButton9")),
                "Couldn't find the stay signed-in button",
            );
            await_with_err_log!(button.click(), "Couldn't click the stay signed-in button");
            Ok(AuthState::Authenticated) // TODO acc check we are authed 
        }
        Unknown => Ok(AuthState::Failure),
        _ => panic!("Todo impl"),
    }
}

async fn determin_2fa_state(driver: &WebDriver) -> Result<Ms2faStates, Error> {
    let first_form = await_with_err_log!(
        driver.query(By::Tag("form")).first(),
        "Couldn't find the authentication form",
    );

    let action = await_with_err_log!(
        first_form.prop("action"),
        "Couldn't read the authentication form action",
    )
    .ok_or_else(|| {
        WebDriverError::NotFound("form action".to_string(), "property was empty".to_string())
    })
    .map_err(|e| {
        if_logging!(eprintln!("The authentication form has no action property"));
        e
    })?;
    dbg::log!("[auth_spool] determin poss form action={:?}", action);

    match action.as_str() {
        "https://login.microsoftonline.com/appverify" => return Ok(Ms2faStates::TrustWebsite),
        "https://login.microsoftonline.com/kmsi" => return Ok(Ms2faStates::StaySignedIn),
        "https://login.microsoftonline.com/common/SAS/ProcessAuth" => {
            let is_notification = driver
                .find(By::Id("idRichContext_DisplaySign"))
                .await
                .is_ok();

            let is_otp = driver.find(By::Id("idTxtBx_SAOTCC_OTC")).await.is_ok();

            return if is_notification {
                Ok(Ms2faStates::PhoneAppNotification)
            } else if is_otp {
                Ok(Ms2faStates::PhoneOTP)
            } else {
                Ok(Ms2faStates::ChooseVerificationOption)
            };
        }

        _ => (),
    }

    Ok(Ms2faStates::Unknown)
}

pub async fn handler_phone_notification(driver: &WebDriver, otc: u64) -> Result<AuthState, Error> {
    dbg::log!("[phone_notif] checking were still waiting");

    if await_with_err_log!(
        determin_2fa_state(driver),
        "Couldn't check the phone notification state",
    ) != Ms2faStates::PhoneAppNotification
    {
        dbg::log!("[phone_notif] Code has been accepted or rejected");
        sleep(Duration::from_secs(1)).await;
        return Ok(AuthState::AuthSpooling);
    }

    sleep(Duration::from_secs(1)).await;
    Ok(AuthState::ApproveAppNotif(otc))
}

pub async fn handler_phone_otp(
    driver: &WebDriver,
    otp: Option<String>,
) -> Result<AuthState, Error> {
    dbg::log!("[phone_otp] Waiting for code input");
    if otp.is_none() {
        dbg::log!(
            "[phone_otp] Not recived a code, should possibly return a failure state instead? Or we can still leave it to the caller"
        );
        return Ok(AuthState::PhoneOTP(None));
    }
    let otp = otp.unwrap();

    //TODO improve the handling here
    let input = await_with_err_log!(
        driver.find(By::Id("idTxtBx_SAOTCC_OTC")),
        "Couldn't find the phone OTP input field",
    );
    await_with_err_log!(input.send_keys(otp), "Couldn't enter the phone OTP");

    let button = await_with_err_log!(
        driver.find(By::Id("idSubmit_SAOTCC_Continue")),
        "Couldn't find the OTP continue button",
    );
    await_with_err_log!(button.click(), "Couldn't click the OTP continue button");

    Ok(AuthState::AuthSpooling)
}
