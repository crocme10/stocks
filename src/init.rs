use clap::ArgMatches;

use stocks::error;

#[allow(clippy::needless_lifetimes)]
pub async fn init<'a>(_matches: &ArgMatches<'a>) -> Result<(), error::Error> {
    Ok(())
}
