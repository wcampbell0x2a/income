use std::io::Write;

use env_logger::Env;
use log::Level;

/// Custom logger
///
/// Default env: RUST_LOG=info
/// INFO - Just stdout
/// OTHER - regular eng_logger
pub fn init() {
    let mut builder = env_logger::Builder::from_env(Env::default().default_filter_or("info"));
    builder
        .format(|buf, record| {
            // Format INFO level messages without any prefix/metadata
            if record.level() == Level::Info {
                writeln!(buf, "{}", record.args())
            } else {
                // Use the default colored formatter for non-INFO messages
                let level = record.level();
                let level_style = buf.default_level_style(level);

                writeln!(
                    buf,
                    "[{}{}{}] {}: {}",
                    level_style,
                    level,
                    level_style.render_reset(),
                    record.target(),
                    record.args()
                )
            }
        })
        .init();
}
