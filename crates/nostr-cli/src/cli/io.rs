// Copyright (c) 2022-2023 Yuki Kishimoto
// Copyright (c) 2023-2024 Rust Nostr Developers
// Distributed under the MIT software license

use dialoguer::{Confirm, Input, Password};
use nostr_sdk::{Keys, Result};

pub fn get_optional_input<S>(prompt: S) -> Result<Option<String>>
where
    S: Into<String>,
{
    let input: String = Input::new()
        .with_prompt(prompt)
        .allow_empty(true)
        .default(String::new())
        .interact_text()?;

    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

pub fn get_keys<S>(prompt: S) -> Result<Keys>
where
    S: Into<String>,
{
    let secret_key = Password::new().with_prompt(prompt).interact()?;
    Ok(Keys::parse(secret_key)?)
}

/* pub fn get_password_with_confirmation() -> Result<String> {
    Ok(Password::new()
        .with_prompt("New password")
        .with_confirmation("Confirm password", "Passwords mismatching")
        .interact()?)
} */

pub fn ask<S>(prompt: S) -> Result<bool>
where
    S: Into<String>,
{
    if Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()?
    {
        Ok(true)
    } else {
        Ok(false)
    }
}
