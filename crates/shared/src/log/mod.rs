mod configuration;
mod console;
mod util;

use crate::config::Config;
use crate::log::configuration::{
    appender_str_to_hashmap, hashmap_to_appender, hashmap_to_logger, logger_str_to_hashmap,
};
use anyhow::Result;
use config::{Map, Value, ValueKind};
use log::{Level, LevelFilter, Log, Metadata, Record};
use log4rs::append::Append;
use log4rs::config::{Appender, Logger, Root};
use std::ffi::{c_char, CStr};
use std::ops::Deref;

const LEVELS: [&str; 6] = ["OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

pub struct LogMgr(log4rs::Logger);

impl LogMgr {
    pub fn new(config: &Config) -> Result<Self> {
        let appenders_config = config.get::<Map<String, Value>>("appender", None)?;
        let appenders = appenders_config.iter().filter_map(|(name, value)| {
            let appender: Result<Box<dyn Append>> = match &value.kind {
                ValueKind::String(config_str) => {
                    appender_str_to_hashmap(config_str.as_str(), value.origin())
                        .and_then(|config_map| hashmap_to_appender(&config_map))
                }
                ValueKind::Table(config_map) => hashmap_to_appender(config_map),
                _ => {
                    return None;
                }
            };

            Some((name, appender))
        });

        let mut appender_list = vec![];
        for (name, appender) in appenders {
            if let Err(error) = appender {
                eprintln!("Error in appender config {}: {}", name, error.to_string());
                continue;
            }

            appender_list.push(Appender::builder().build(name, appender?));
        }

        let loggers_config = config.get::<Map<String, Value>>("logger", None)?;
        let loggers = loggers_config.iter().filter_map(|(name, value)| {
            let logger: Result<Logger> = match &value.kind {
                ValueKind::String(config_str) => {
                    logger_str_to_hashmap(config_str.as_str(), value.origin())
                        .and_then(|h| hashmap_to_logger(name, &h))
                }
                ValueKind::Table(config_map) => hashmap_to_logger(name, config_map),
                _ => return None,
            };

            Some((name, logger))
        });

        let mut logger_list = vec![];
        let mut root = None;
        for (name, logger) in loggers {
            if let Err(error) = logger {
                eprintln!("Error in logger config {}: {}", name, error.to_string());
                continue;
            }

            let logger = logger.unwrap();
            if name.as_str() == "root" {
                root = Some(
                    Root::builder()
                        .appenders(logger.appenders())
                        .build(logger.level()),
                );
            } else {
                logger_list.push(logger);
            }
        }

        let config = ::log4rs::config::Config::builder()
            .appenders(appender_list)
            .loggers(logger_list)
            .build(root.unwrap_or_else(|| Root::builder().build(LevelFilter::Warn)))?;

        Ok(Self(log4rs::Logger::new(config)))
    }

    /// Write a log entry
    ///
    /// # Safety
    /// C FFI: char pointers are checked for equality to nullptr.
    #[no_mangle]
    #[cfg(feature = "ffi_log")]
    pub unsafe extern "C" fn LogMgr_Write(
        &self,
        target: *const c_char,
        level: u8,
        message: *const c_char,
    ) {
        if target.is_null() || message.is_null() {
            return;
        }

        let level = match level {
            1 => Level::Trace,
            2 => Level::Debug,
            3 => Level::Info,
            4 => Level::Warn,
            5 => Level::Error,
            _ => return,
        };

        let target = CStr::from_ptr(target).to_str().unwrap().replace('.', "::");
        let metadata = Metadata::builder().target(&target).level(level).build();

        if !self.enabled(&metadata) {
            return;
        }

        let message = CStr::from_ptr(message).to_str().unwrap();
        self.log(
            &Record::builder()
                .metadata(metadata)
                .args(format_args!("{}", message))
                .build(),
        );

        if target == "server::loading" {
            // Show startup logs
            self.flush();
        }
    }
}

impl Log for LogMgr {
    #[inline]
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.0.enabled(metadata)
    }

    #[inline]
    fn log(&self, record: &Record) {
        self.0.log(record)
    }

    #[inline]
    fn flush(&self) {
        Log::flush(&self.0)
    }
}

#[no_mangle]
#[cfg(feature = "ffi_log")]
pub extern "C" fn LogMgr_Initialize(config: &Config) -> *const LogMgr {
    let mgr = LogMgr::new(config);
    let Ok(log_mgr) = mgr else {
        eprintln!("Cannot initialize log manager: {}", unsafe { mgr.unwrap_err_unchecked() }.to_string());
        return std::ptr::null();
    };

    let boxed = Box::new(log_mgr);
    let pointer = Box::leak(boxed);
    let res = ::log::set_logger(pointer);
    if let Err(e) = res {
        eprintln!(
            "Cannot set log manager as default logger: {}",
            e.to_string()
        );
    }

    ::log::set_max_level(LevelFilter::Trace);
    pointer.deref()
}

#[no_mangle]
#[cfg(feature = "ffi_log")]
pub extern "C" fn LogMgr_Free(logmgr: *mut LogMgr) {
    let b = unsafe { Box::<LogMgr>::from_raw(logmgr as *mut _) };
    drop(b)
}
