//! Engine protocol logging.
//!
//! Logs incoming commands and outgoing responses with timestamps.
//! Zero overhead when disabled (writer is None).
//! Enable: `setoption name LogFile value <path>`
//! Disable: `setoption name LogFile value none`

use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

/// Protocol logger. Wraps an optional file writer.
pub struct ProtocolLog {
    writer: Option<BufWriter<File>>,
    start: Instant,
}

impl ProtocolLog {
    /// Create a new disabled protocol logger.
    pub fn new() -> Self {
        Self {
            writer: None,
            start: Instant::now(),
        }
    }

    /// Enable logging to a file. Pass "none" to disable.
    pub fn set_logfile(&mut self, path: &str) -> std::io::Result<()> {
        if path == "none" || path.is_empty() {
            self.writer = None;
            return Ok(());
        }

        let file = File::create(path)?;
        self.writer = Some(BufWriter::new(file));
        self.start = Instant::now();
        Ok(())
    }

    /// Log an incoming command.
    pub fn log_incoming(&mut self, command: &str) {
        self.write_entry('>', command);
    }

    /// Log an outgoing response.
    pub fn log_outgoing(&mut self, response: &str) {
        self.write_entry('<', response);
    }

    /// Check if logging is enabled.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.writer.is_some()
    }

    fn write_entry(&mut self, prefix: char, message: &str) {
        if let Some(ref mut writer) = self.writer {
            let elapsed = self.start.elapsed();
            let secs = elapsed.as_secs_f64();
            // Ignore write errors — logging should never crash the engine
            let _ = writeln!(writer, "[{:09.3}] {} {}", secs, prefix, message);
            let _ = writer.flush();
        }
    }
}

impl Default for ProtocolLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_disabled_by_default() {
        let log = ProtocolLog::new();
        assert!(!log.is_enabled());
    }

    #[test]
    fn test_log_to_file() {
        let dir = std::env::temp_dir();
        let path = dir.join("valhalla_test_protocol.log");
        let path_str = path.to_str().unwrap();

        let mut log = ProtocolLog::new();
        log.set_logfile(path_str).unwrap();
        assert!(log.is_enabled());

        log.log_incoming("uci");
        log.log_outgoing("id name Valhalla");
        log.log_outgoing("uciok");

        // Disable to flush
        log.set_logfile("none").unwrap();
        assert!(!log.is_enabled());

        // Read and verify
        let mut contents = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        assert!(
            contents.contains("> uci"),
            "Should contain incoming command"
        );
        assert!(
            contents.contains("< id name Valhalla"),
            "Should contain outgoing response"
        );
        assert!(
            contents.contains("< uciok"),
            "Should contain uciok response"
        );

        // Clean up
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_timestamp_format() {
        let dir = std::env::temp_dir();
        let path = dir.join("valhalla_test_timestamp.log");
        let path_str = path.to_str().unwrap();

        let mut log = ProtocolLog::new();
        log.set_logfile(path_str).unwrap();
        log.log_incoming("test");
        log.set_logfile("none").unwrap();

        let mut contents = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut contents)
            .unwrap();

        // Should match pattern [000.000] > test
        assert!(
            contents.starts_with('['),
            "Should start with timestamp bracket"
        );
        assert!(contents.contains("] > test"), "Should have correct format");

        let _ = std::fs::remove_file(&path);
    }
}
