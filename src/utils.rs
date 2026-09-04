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
