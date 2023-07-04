const BANNER: &str = r#"
 __  __      __
/\ \/\ \  __/\ \__
\ \ \/'/'/\_\ \ ,_\  _ __   ___     ___
 \ \ , < \/\ \ \ \/ /\`'__\/ __`\ /' _ `\
  \ \ \\`\\ \ \ \ \_\ \ \//\ \L\ \/\ \/\ \
   \ \_\ \_\ \_\ \__\\ \_\\ \____/\ \_\ \_\
    \/_/\/_/\/_/\/__/ \/_/ \/___/  \/_/\/_/
"#;

pub fn get_banner() -> &'static str {
    BANNER
}

/// Print out the banner to stdout
#[no_mangle]
pub extern "C" fn PrintBanner() {
    println!("{}", BANNER);
}
