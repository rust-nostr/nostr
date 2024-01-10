use nostr::Result;

pub mod keys;

fn main() -> Result<()> {
    keys::keys()?;
    Ok(())
}
