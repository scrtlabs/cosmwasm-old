use log::{Metadata, Record};

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // Not really needed since we set logging level at lib.rs in the init function
        true
    }

    fn log(&self, record: &Record) {
        println!(
            "{}  [{}] {}",
            record.level(),
            record.target(),
            record.args()
        );
    }

    fn flush(&self) {}
}
