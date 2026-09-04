use std::future::Future;

use serde_json::json;
use thirtyfour::prelude::*;

use crate::dbg;
use crate::disable_webauth::get_extension_path;

macro_rules! if_logging {
    ($($code:tt)*) => {
        #[cfg(not(feature = "no-logging"))]
        {
            $($code)*
        }
    };
}
pub(crate) use if_logging;

pub(crate) async fn await_with_error_logging_async<T, F>(
    operation: F,
    message: &str,
) -> Result<T, WebDriverError>
where
    F: Future<Output = Result<T, WebDriverError>>,
{
    operation.await.map_err(|error| {
        if_logging!(eprintln!("{}", message));
        error
    })
}

macro_rules! await_with_err_log {
    ($operation:expr, $message:expr $(,)?) => {{
        match $crate::utils::await_with_error_logging_async($operation, $message).await {
            Ok(value) => value,
            Err(error) => return Err(error),
        }
    }};
}
pub(crate) use await_with_err_log;

pub async fn create_driver() -> Result<WebDriver, WebDriverError> {
    let mut caps = DesiredCapabilities::chrome(); //TODO: See if we can auth find a browser to skip

    let (ext_path, _guard) =
        get_extension_path().expect("Could not create dissable webauth plugin"); // TODO
    // remove
    // exepct
    dbg::log!("[init] Extension path: {}", ext_path.display());
    caps.add_arg(&format!("--load-extension={}", ext_path.display()))?;

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

pub async fn cookies_from(driver: &WebDriver, sites: Vec<&str>) -> String {
    let mut all_cookies = Vec::new();

    for s in sites {
        if let Ok(_) = driver.goto(s).await {
            if let Ok(cookies) = driver.get_all_cookies().await {
                all_cookies.extend(cookies);
            }
        }
    }

    serde_json::to_string(&all_cookies).unwrap()
}

pub async fn current_page_cookies<'a>(driver: &WebDriver) -> String {
    #[cfg(not(feature = "dbg"))]
    return serde_json::to_string(&json!(driver.get_all_cookies().await.unwrap())).unwrap();

    #[cfg(feature = "dbg")]
    return serde_json::to_string_pretty(&json!(driver.get_all_cookies().await.unwrap())).unwrap();
}
