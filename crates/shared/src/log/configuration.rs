use super::console::ConsoleAppender;
use super::LEVELS;
use crate::log::console::Target;
use anyhow::{Error, Result};
use config::Value;
use console::{Color, Style};
use log::{Level, LevelFilter};
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::Policy;
use log4rs::append::rolling_file::{LogFile, RollingFileAppender};
use log4rs::append::Append;
use log4rs::config::Logger;
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug)]
struct NopPolicy;
impl Policy for NopPolicy {
    fn process(&self, _: &mut LogFile) -> Result<()> {
        Ok(())
    }
}

pub(super) fn appender_str_to_hashmap(
    conf: &str,
    origin: Option<&str>,
) -> Result<HashMap<String, Value>> {
    let origin = origin.map(|o| o.to_string());

    let mut result = HashMap::default();
    let config = conf.split(',').map(|s| s.trim()).collect::<Vec<_>>();
    let ty = match config.first() {
        Some(&"1") => "console",
        Some(&"2") => "file",
        v => {
            return Err(Error::msg(format!(
                "Invalid appender type {}",
                v.cloned().unwrap_or_default()
            )))
        }
    };
    result.insert(
        "type".to_string(),
        Value::new(origin.as_ref(), ty.to_string()),
    );

    let level = match config.get(1) {
        Some(&"0") | Some(&"off") => "OFF",
        Some(&"1") | Some(&"trace") => "TRACE",
        None | Some(&"2") | Some(&"debug") => "DEBUG",
        Some(&"3") | Some(&"info") => "INFO",
        Some(&"4") | Some(&"warn") => "WARN",
        Some(&"5") | Some(&"error") => "ERROR",
        level => {
            return Err(Error::msg(format!(
                r#"Invalid level "{}" specified"#,
                level.cloned().unwrap_or_default(),
            )));
        }
    };

    result.insert(
        "level".to_string(),
        Value::new(origin.as_ref(), level.to_string()),
    );

    let opt1 = config.get(2);
    let opt2 = config.get(3);

    if ty == "console" {
        if let Some(colors) = opt1.cloned() {
            let mut color_map = HashMap::default();
            for (level, color) in colors.split(' ').enumerate() {
                let Some(level) = LEVELS.get(level + 1).map(|l| l.to_string()) else { continue; };
                let Ok(color) = color.parse::<u8>() else { continue; };

                color_map.insert(level, color);
            }

            result.insert(
                "color_map".to_string(),
                Value::new(origin.as_ref(), color_map),
            );
        }

        result.insert(
            "target".to_string(),
            Value::new(
                origin.as_ref(),
                match opt2 {
                    Some(&"e") => "stderr",
                    Some(&"o") | None => "stdout",
                    t => {
                        return Err(Error::msg(format!(
                            r#"Invalid target "{}""#,
                            t.cloned().unwrap_or_default()
                        )))
                    }
                },
            ),
        );
    } else {
        let Some(filename) = opt1.map(|o| o.to_string()) else {
            return Err(Error::msg("No filename has been specified"));
        };
        result.insert(
            "filename".to_string(),
            Value::new(origin.as_ref(), filename),
        );

        let append = match opt2 {
            None | Some(&"a") => true,
            Some(&"w") => false,
            _ => {
                return Err(Error::msg(format!(
                    r#"Invalid file mode "{}" specified"#,
                    opt2.cloned().unwrap_or("")
                )))
            }
        };
        result.insert("append".to_string(), Value::new(origin.as_ref(), append));

        if let Some(max_file_size) = config.get(4).cloned() {
            if let Ok(max_file_size) = max_file_size.parse::<i64>() {
                result.insert(
                    "max_file_size".to_string(),
                    Value::new(origin.as_ref(), max_file_size),
                );
            }
        }
    }

    Ok(result)
}

pub(super) fn hashmap_to_appender(config: &HashMap<String, Value>) -> Result<Box<dyn Append>> {
    let Some(ty) = config.get("type") else {
        return Err(Error::msg(r#"Missing mandatory config key "type" in appender"#));
    };

    let ty = ty.clone().into_string()?.to_lowercase();
    match ty.as_str() {
        "console" => hashmap_console_to_appender(config),
        "file" => hashmap_file_to_appender(config),
        t => Err(Error::msg(format!(
            r#"Invalid config value for key "type" in appender. Accepted values are "file" and "console", "{}" passed."#,
            t
        ))),
    }
}

fn hashmap_console_to_appender(config: &HashMap<String, Value>) -> Result<Box<dyn Append>> {
    let mut builder = ConsoleAppender::builder();
    if let Some(level) = config.get("level") {
        let level = level.clone().into_string()?;
        builder = builder.log_level(Level::from_str(&level)?);
    } else {
        builder = builder.log_level(Level::Debug);
    }

    if let Some(Ok(colors)) = config.get("color_map").map(|v| v.clone().into_table()) {
        for (level, color_code) in colors {
            let Ok(level) = Level::from_str(&level) else { continue; };
            let Ok(color_code) = color_code.into_uint() else { continue; };
            let Ok(color_code) = u8::try_from(color_code) else { continue; };

            let style = Style::new().fg(Color::Color256(color_code));
            builder = builder.set_level_style(level, style);
        }
    }

    if let Some(target) = config.get("target") {
        if let Ok(target) = target.clone().into_string() {
            let lower_target = target.to_lowercase();
            match lower_target.as_str() {
                "stdout" => builder = builder.target(Target::Stdout),
                "stderr" => builder = builder.target(Target::Stderr),
                t => {
                    return Err(Error::msg(format!(
                        r#"Invalid target provided. Accepted values are "stdout" and "stderr". "{}" passed."#,
                        t
                    )))
                }
            }
        } else {
            builder = builder.target(Target::Stdout);
        }
    }

    Ok(Box::new(builder.build()))
}

fn hashmap_file_to_appender(config: &HashMap<String, Value>) -> Result<Box<dyn Append>> {
    let filename = config
        .get("filename")
        .ok_or(Error::msg("No filename has been provided for appender"))?;
    let filename = filename.clone().into_string()?;
    let append = config
        .get("append")
        .cloned()
        .map(|v| v.into_bool())
        .unwrap_or(Ok(true))?;
    let max_file_size = config.get("max_file_size").cloned().map(|v| v.into_uint());

    let mut builder = RollingFileAppender::builder();
    builder = builder.append(append);

    let policy: Box<dyn Policy> = if let Some(Ok(max_file_size)) = max_file_size {
        Box::new(CompoundPolicy::new(
            Box::new(SizeTrigger::new(max_file_size)),
            {
                let file_path = Path::new(&filename);
                let Some(base_file_path) = file_path.file_stem() else { return Err(Error::msg("Invalid filename provided")); };
                let base_file_path = base_file_path.to_str().map(|s| s.to_string()).unwrap();
                let extension = file_path
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "log".to_string());

                let pattern = format!("{}.{}.{}", base_file_path, "{}", extension);
                Box::new(FixedWindowRoller::builder().build(&pattern, 10)?)
            },
        ))
    } else {
        Box::new(NopPolicy {})
    };

    Ok(Box::new(builder.build(filename, policy)?))
}

pub(super) fn hashmap_to_logger(name: &str, config: &HashMap<String, Value>) -> Result<Logger> {
    let level_filter = if let Some(level) = config.get("level") {
        let level = level.clone().into_string()?;
        LevelFilter::from_str(&level)?
    } else {
        LevelFilter::Debug
    };

    let appender_list = config
        .get("appenders")
        .ok_or_else(|| Error::msg("Missing mandatory appenders"))?
        .clone();
    let appender_list = appender_list.into_array()?;
    if appender_list.is_empty() {
        return Err(Error::msg("Appender list is empty"));
    }

    let mut appenders = vec![];
    for appender in appender_list {
        let appender = appender.into_string()?.trim().to_lowercase();
        appenders.push(appender);
    }

    Ok(Logger::builder()
        .appenders(appenders)
        .build(name, level_filter))
}

pub(super) fn logger_str_to_hashmap(
    config: &str,
    origin: Option<&str>,
) -> Result<HashMap<String, Value>> {
    let origin = origin.map(|o| o.to_string());
    let config = config.split(',').collect::<Vec<_>>();
    let mut result = HashMap::default();

    let level = match config.first() {
        Some(&"0") | Some(&"off") => "OFF",
        Some(&"1") | Some(&"trace") => "TRACE",
        None | Some(&"2") | Some(&"debug") => "DEBUG",
        Some(&"3") | Some(&"info") => "INFO",
        Some(&"4") | Some(&"warn") => "WARN",
        Some(&"5") | Some(&"error") => "ERROR",
        level => {
            return Err(Error::msg(format!(
                r#"Invalid level "{}" specified"#,
                level.cloned().unwrap_or_default(),
            )));
        }
    };

    result.insert("level".to_string(), Value::new(origin.as_ref(), level));

    let appender_list = config.get(1);
    let Some(appender_list) = appender_list else {
        return Err(Error::msg("Appender list is empty"));
    };

    let appender_list = appender_list
        .split(' ')
        .filter(|a| !a.trim().is_empty())
        .map(|a| a.to_lowercase())
        .collect::<Vec<_>>();
    if appender_list.is_empty() {
        return Err(Error::msg("Appender list is empty"));
    }

    result.insert(
        "appenders".to_string(),
        Value::new(origin.as_ref(), appender_list),
    );

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::log::configuration::{
        appender_str_to_hashmap, hashmap_to_appender, hashmap_to_logger, logger_str_to_hashmap,
    };
    use anyhow::Result;
    use config::Value;
    use std::collections::HashMap;

    #[test]
    pub fn simple_hashmap_to_appender() -> Result<()> {
        let config = HashMap::from([("type".to_string(), Value::new(None, "console"))]);
        assert!(hashmap_to_appender(&config).is_ok());

        let config = HashMap::from([
            ("type".to_string(), Value::new(None, "file")),
            ("filename".to_string(), Value::new(None, "log_file.log")),
        ]);
        assert!(hashmap_to_appender(&config).is_ok());

        let config = HashMap::from([(
            "not_even_an_option".to_string(),
            Value::new(None, "it's a trap!"),
        )]);
        assert!(hashmap_to_appender(&config).is_err());

        let config = HashMap::from([("type".to_string(), Value::new(None, "it's a trap!"))]);
        assert!(hashmap_to_appender(&config).is_err());

        Ok(())
    }

    #[test]
    pub fn appender_console_simple_str_to_hashmap() -> Result<()> {
        let map = appender_str_to_hashmap("1,3,,e", None)?;
        assert_eq!(
            map,
            HashMap::from([
                ("type".to_string(), Value::new(None, "console")),
                ("level".to_string(), Value::new(None, "INFO")),
                (
                    "color_map".to_string(),
                    Value::new(None, HashMap::<String, u8>::default())
                ),
                ("target".to_string(), Value::new(None, "stderr")),
            ])
        );

        let map = appender_str_to_hashmap("1", None)?;
        assert_eq!(
            map,
            HashMap::from([
                ("type".to_string(), Value::new(None, "console")),
                ("level".to_string(), Value::new(None, "DEBUG")),
                ("target".to_string(), Value::new(None, "stdout")),
            ])
        );

        let map = appender_str_to_hashmap("1,warn", None)?;
        assert_eq!(
            map,
            HashMap::from([
                ("type".to_string(), Value::new(None, "console")),
                ("level".to_string(), Value::new(None, "WARN")),
                ("target".to_string(), Value::new(None, "stdout")),
            ])
        );

        let map = appender_str_to_hashmap("1,trace,9 11 12 5 2,o", None)?;
        assert_eq!(
            map,
            HashMap::from([
                ("type".to_string(), Value::new(None, "console")),
                ("level".to_string(), Value::new(None, "TRACE")),
                (
                    "color_map".to_string(),
                    Value::new(
                        None,
                        HashMap::from([
                            ("ERROR".to_string(), 9_u8),
                            ("WARN".to_string(), 11_u8),
                            ("INFO".to_string(), 12_u8),
                            ("DEBUG".to_string(), 5_u8),
                            ("TRACE".to_string(), 2_u8),
                        ])
                    )
                ),
                ("target".to_string(), Value::new(None, "stdout")),
            ])
        );

        let e = appender_str_to_hashmap("1,12", None).expect_err("Expected error");
        assert_eq!(e.to_string(), r#"Invalid level "12" specified"#);

        let e = appender_str_to_hashmap("1,not_a_level", None).expect_err("Expected error");
        assert_eq!(e.to_string(), r#"Invalid level "not_a_level" specified"#);

        let e = appender_str_to_hashmap("1,2,,x", None).expect_err("Expected error");
        assert_eq!(e.to_string(), r#"Invalid target "x""#);

        Ok(())
    }

    #[test]
    pub fn appender_file_simple_str_to_hashmap() -> Result<()> {
        let map = appender_str_to_hashmap("2,error,file_error.log", None)?;
        assert_eq!(
            map,
            HashMap::from([
                ("type".to_string(), Value::new(None, "file")),
                ("level".to_string(), Value::new(None, "ERROR")),
                ("filename".to_string(), Value::new(None, "file_error.log")),
                ("append".to_string(), Value::new(None, true)),
            ])
        );

        let map = appender_str_to_hashmap("2,4,file_error.log,w", None)?;
        assert_eq!(
            map,
            HashMap::from([
                ("type".to_string(), Value::new(None, "file")),
                ("level".to_string(), Value::new(None, "WARN")),
                ("filename".to_string(), Value::new(None, "file_error.log")),
                ("append".to_string(), Value::new(None, false)),
            ])
        );

        let map = appender_str_to_hashmap("2,4,file_error.log,w,1024000", None)?;
        assert_eq!(
            map,
            HashMap::from([
                ("type".to_string(), Value::new(None, "file")),
                ("level".to_string(), Value::new(None, "WARN")),
                ("filename".to_string(), Value::new(None, "file_error.log")),
                ("append".to_string(), Value::new(None, false)),
                ("max_file_size".to_string(), Value::new(None, 1024000)),
            ])
        );

        let e = appender_str_to_hashmap("2,2", None).expect_err("Expected error");
        assert_eq!(e.to_string(), r#"No filename has been specified"#);

        let e = appender_str_to_hashmap("2,2,file.log,x", None).expect_err("Expected error");
        assert_eq!(e.to_string(), r#"Invalid file mode "x" specified"#);

        let e = appender_str_to_hashmap("2,12,file.log", None).expect_err("Expected error");
        assert_eq!(e.to_string(), r#"Invalid level "12" specified"#);

        let e =
            appender_str_to_hashmap("2,not_a_level,file.log", None).expect_err("Expected error");
        assert_eq!(e.to_string(), r#"Invalid level "not_a_level" specified"#);

        Ok(())
    }

    #[test]
    pub fn simple_hashmap_to_logger() -> Result<()> {
        let config = HashMap::from([(
            "appenders".to_string(),
            Value::new(None, vec!["console".to_string()]),
        )]);
        assert!(hashmap_to_logger("test", &config).is_ok());

        let config = HashMap::from([
            ("level".to_string(), Value::new(None, "DEBUG")),
            (
                "appenders".to_string(),
                Value::new(None, vec!["console".to_string()]),
            ),
        ]);
        assert!(hashmap_to_logger("test", &config).is_ok());

        let config = HashMap::from([(
            "not_even_an_option".to_string(),
            Value::new(None, "it's a trap!"),
        )]);
        assert!(hashmap_to_logger("test", &config).is_err());

        let config = HashMap::from([("level".to_string(), Value::new(None, "it's a trap!"))]);
        assert!(hashmap_to_logger("test", &config).is_err());

        let config = HashMap::from([(
            "appenders".to_string(),
            Value::new(None, Vec::<String>::default()),
        )]);
        assert!(hashmap_to_logger("test", &config).is_err());

        Ok(())
    }

    #[test]
    pub fn logger_simple_str_to_hashmap() -> Result<()> {
        let e = logger_str_to_hashmap("12", None).expect_err("Expected error");
        assert_eq!(e.to_string(), r#"Invalid level "12" specified"#);

        let e = logger_str_to_hashmap("not_a_level", None).expect_err("Expected error");
        assert_eq!(e.to_string(), r#"Invalid level "not_a_level" specified"#);

        let e = logger_str_to_hashmap("2", None).expect_err("Expected error");
        assert_eq!(e.to_string(), "Appender list is empty");

        let e = logger_str_to_hashmap("2,", None).expect_err("Expected error");
        assert_eq!(e.to_string(), "Appender list is empty");

        let map = logger_str_to_hashmap("3,Console Auth", None)?;
        assert_eq!(
            map,
            HashMap::from([
                ("level".to_string(), Value::new(None, "INFO")),
                (
                    "appenders".to_string(),
                    Value::new(None, vec!["console".to_string(), "auth".to_string()])
                ),
            ])
        );

        Ok(())
    }
}
