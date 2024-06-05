use anyhow::Result;

fn main() -> Result<()> {
    wit_deps::lock_sync!()?;
    Ok(())
}
