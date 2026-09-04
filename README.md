# uoe-ms-auth

`uoe-ms-auth` automates the University of Edinburgh Microsoft authentication, it can be 
interfaced by:
- Rust 
- Python
- more to come


## Command line application

The libary can be ran directly if no bindings are available (if being used inside of another project its recommended to disable the logging feature)

```bash
cargo run --release
```

## Rust library

To just hook the existing binary's flow use:

```rust
let cookies = uoe_ms_auth::run().await;
```

If you're running a GUI or for any other reason don't want to use hte existing inputs,
use the callback-driven runner (you may also want to disable the logging feature): 

```rust
let cookies = uoe_ms_auth::run_with_credentials(
    username,
    password,
    || prompt_for_otp().map_err(|error| error.to_string()),
    |number| {
        notify_user_to_approve(number);
        Ok(())
    },
)
.await?;
```

The OTP callback is called only when a OTP is required from the user. The approval
callback is called if the user needs to accept a given prompt on Microsoft Authenticator
it is the preferred method of 2fa by the program with OTP being a fallback if its the only option

## Python bindings

Install the bindings with
```bash
pip install uoe_ms_auth
```

After installation, authenticate from Python using the equivalent of run with credentials

```python
import uoe_ms_auth

cookies = uoe_ms_auth.authenticate(
    username="s1234567@ed.ac.uk",
    password="your-password",
    otp_callback=lambda: input("Authenticator code: "),
    approval_callback=lambda code: print(f"Approve code: {code}"),
)
```

`authenticate` returns the authenticated cookies as a JSON string. The
`otp_callback` must return the OTP as a string. The `approval_callback` receives
the Authenticator number as an integer. 

At the moment the python bindings are relatively limited however this is being worked on

## Build features

`logging` - Default feature that prints errors and what the program is doing
`show-browser` - Disables headless browser mode and shows what's happening
`dbg` - Shows verbose debugging output

## Documentation

The authentication flow is described in [`docs/authflow.md`](docs/authflow.md).
