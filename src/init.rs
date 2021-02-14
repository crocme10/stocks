use clap::ArgMatches;
use slog::{o, Drain};

use stocks::db;
use stocks::error;
use stocks::settings::Settings;

#[allow(clippy::needless_lifetimes)]
pub async fn init<'a>(matches: &ArgMatches<'a>) -> Result<(), error::Error> {
    let settings = Settings::new(matches)?;

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!());

    db::pg::init_db(&settings.database.url, logger).await
}
