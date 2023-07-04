use console::{Style, Term};
use log::{Level, Record};
use std::collections::HashMap;
use std::fmt::Debug;
use std::io;
use std::io::Write;
use std::ops::DerefMut;
use std::sync::Mutex;

use log4rs::append::Append;
use log4rs::encode;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::encode::Encode;
use crate::log::util::BufWriter;

#[derive(Debug)]
struct TermWriter(Term);

impl Write for TermWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl Drop for TermWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

impl encode::Write for TermWriter {}

/// An appender which logs to standard out.
///
/// It supports output styling if standard out is a console buffer on Windows
/// or is a TTY on Unix.
#[derive(Debug)]
pub struct ConsoleAppender {
    writer: Mutex<TermWriter>,
    encoder: Box<dyn Encode>,
    log_level: Level,
    level_style_map: HashMap<Level, Style>,
    is_tty: bool,
    do_write: bool,
}

impl Append for ConsoleAppender {
    fn append(&self, record: &Record) -> anyhow::Result<()> {
        let level = record.metadata().level();
        if self.do_write && level <= self.log_level {
            if self.is_tty {
                let mut writer = BufWriter::default();
                self.encoder.encode(&mut writer, record)?;
                let _ = writer.flush();

                let style = self
                    .level_style_map
                    .get(&level)
                    .cloned()
                    .unwrap_or_default();
                let styled = style.apply_to(String::from_utf8_lossy(writer.buffer()));

                let mut w = self.writer.lock().unwrap();
                write!(w.deref_mut(), "{}", styled)?;
            } else {
                let mut w = self.writer.lock().unwrap();
                self.encoder.encode(w.deref_mut(), record)?;
            }
        }

        Ok(())
    }

    fn flush(&self) {
        let mut w = self.writer.lock().unwrap();
        let _ = w.flush();
    }
}

impl ConsoleAppender {
    /// Creates a new `ConsoleAppender` builder.
    pub fn builder() -> ConsoleAppenderBuilder {
        ConsoleAppenderBuilder {
            encoder: None,
            target: Target::Stdout,
            log_level: Level::Debug,
            level_style_map: Default::default(),
            tty_only: false,
        }
    }
}

/// A builder for `ConsoleAppender`s.
pub struct ConsoleAppenderBuilder {
    encoder: Option<Box<dyn Encode>>,
    target: Target,
    log_level: Level,
    level_style_map: HashMap<Level, Style>,
    tty_only: bool,
}

impl ConsoleAppenderBuilder {
    /// Sets the output encoder for the `ConsoleAppender`.
    ///
    /// Default: no encoder is set.
    #[allow(dead_code)]
    pub fn encoder(mut self, encoder: Box<dyn Encode>) -> Self {
        self.encoder = Some(encoder);
        self
    }

    /// Sets the minimum log level.
    ///
    /// Defaults to `Level::Debug`.
    #[allow(dead_code)]
    pub fn log_level(mut self, level: Level) -> Self {
        self.log_level = level;
        self
    }

    /// Sets the output stream to log to.
    ///
    /// Defaults to `Target::Stdout`.
    pub fn target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }

    /// Sets the output to log only when it's a TTY.
    ///
    /// Defaults to `false`.
    #[allow(dead_code)]
    pub fn tty_only(mut self, tty_only: bool) -> Self {
        self.tty_only = tty_only;
        self
    }

    /// Sets the style for the given log level.
    pub fn set_level_style(mut self, level: Level, style: Style) -> Self {
        self.level_style_map.insert(level, style);
        self
    }

    /// Consumes the `ConsoleAppenderBuilder`, producing a `ConsoleAppender`.
    pub fn build(self) -> ConsoleAppender {
        let writer = match self.target {
            Target::Stderr => Term::buffered_stderr(),
            Target::Stdout => Term::buffered_stdout(),
        };

        let is_tty = writer.is_term();
        let do_write = is_tty || !self.tty_only;

        ConsoleAppender {
            writer: Mutex::new(TermWriter(writer)),
            log_level: self.log_level,
            level_style_map: self.level_style_map,
            encoder: self
                .encoder
                .unwrap_or_else(|| Box::<PatternEncoder>::default()),
            is_tty,
            do_write,
        }
    }
}

/// The stream to log to.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Target {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}
