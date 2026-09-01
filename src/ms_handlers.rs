use crate::{
    auth_flow::{AuthState, Error},
    dbg, if_logging,
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
    GetCodeFromPhone,
    IncorrectCode,
    StaySignedIn,
    Unknown,
}

/// TODO go through and add acc error handling into all the ms auth code

pub async fn handler_auth_spooling(driver: &WebDriver) -> Result<AuthState, Error> {
    use Ms2faStates::*;
    dbg::log!("[auth_spool] auth spooling");

    match determin_2fa_state(driver).await? {
        TrustWebsite => {
            dbg::log!("[auth_spool][trust_site] trusting ed.ac.uk");
            driver
                .find(By::Id("idSIButton9"))
                .await
                .map_err(|e| {
                    if_logging!(eprintln!("Couldn't find the trust ed.ac.uk button"));
                    e
                })?
                .click()
                .await?;
            sleep(Duration::from_millis(250)).await; // give time for js to update
            Ok(AuthState::AuthSpooling)
        }
        ChooseVerificationOption => {
            dbg::log!("[auth_spool][choose_ver_opt] init");
            //todo check we can acc do phone otp
            driver
                .find(By::XPath("//div[@data-value='PhoneAppNotification']"))
                .await?
                .click()
                .await?;
            dbg::log!("[auth_spool][choose_ver_opt] Clicked PhoneAppOTP");
            sleep(Duration::from_secs(5)).await;
            Ok(AuthState::AuthSpooling)
        }
        PhoneAppNotification => {
            dbg::log!("[auth_spool][phone notif] reading code ");
            let approval_code = driver
                .find(By::Id("idRichContext_DisplaySign"))
                .await?
                .text()
                .await?;
            dbg::log!("[auth_spool][phone notif] approval_code={}", approval_code);
            Ok(AuthState::ApproveAppNotif(
                approval_code
                    .parse::<u64>()
                    .expect("Todo handle if not number"),
            ))
        }
        StaySignedIn => {
            dbg::log!("[auth_spool][stay_singed_in] telling it to stay signed in");
            driver.find(By::Id("idSIButton9")).await?.click().await?;
            Ok(AuthState::Authenticated) // TODO acc check we are authed 
        }
        Unknown => Ok(AuthState::Failure),
        _ => panic!("Todo impl"),
    }
}

async fn determin_2fa_state(driver: &WebDriver) -> Result<Ms2faStates, Error> {
    let first_form = driver.query(By::Tag("form")).first().await?;

    let action = first_form
        .prop("action")
        .await
        .expect("TODO should be more resilient")
        .expect("TODO should be more resilient");
    dbg::log!("[auth_spool] determin poss form action={:?}", action);

    match action.as_str() {
        "https://login.microsoftonline.com/appverify" => return Ok(Ms2faStates::TrustWebsite),
        "https://login.microsoftonline.com/kmsi" => return Ok(Ms2faStates::StaySignedIn),
        "https://login.microsoftonline.com/common/SAS/ProcessAuth" => {
            let is_displaying_code = driver
                .find(By::Id("idRichContext_DisplaySign"))
                .await
                .is_ok();

            return if is_displaying_code {
                Ok(Ms2faStates::PhoneAppNotification)
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

    if determin_2fa_state(driver).await? != Ms2faStates::PhoneAppNotification {
        dbg::log!("Code has been accepted or rejected");
        sleep(Duration::from_secs(1)).await;
        return Ok(AuthState::AuthSpooling);
    }

    sleep(Duration::from_secs(1)).await;
    Ok(AuthState::ApproveAppNotif(otc))
}

pub async fn handler_awaiting_phone_code(
    driver: &WebDriver,
    code_from_phone: Option<u64>,
) -> Result<AuthState, Error> {
    dbg::log!("Waiting for code input");
    if code_from_phone.is_none() {
        dbg::log!(
            "[await_phone_code] Not recived a code, should possibly return a failure state instead? Or we can still leave it to the caller"
        );
        return Ok(AuthState::AwaitingPhoneCode(None));
    }

    Ok(AuthState::Failure)
}
