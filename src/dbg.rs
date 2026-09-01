use thirtyfour::prelude::*;

#[cfg(feature = "dbg")]
macro_rules! log {
    ($($arg:tt)*) => {
        eprintln!($($arg)*)
    };
}
pub(crate) use log;

#[cfg(not(feature = "dbg"))]
macro_rules! log {
    ($($arg:tt)*) => {};
}

pub async fn current_url(driver: &WebDriver) {
    println!(
        "{}",
        driver
            .current_url()
            .await
            .expect("Failed to get current_url")
    );
}

pub async fn current_content(driver: &WebDriver) {
    println!(
        "{}",
        driver.source().await.expect("Failed to get page source")
    );
}
