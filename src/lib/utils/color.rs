const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";

pub fn green(enabled: bool, text: String) -> String {
    wrap(enabled, GREEN, text)
}

pub fn yellow(enabled: bool, text: String) -> String {
    wrap(enabled, YELLOW, text)
}

pub fn red(enabled: bool, text: String) -> String {
    wrap(enabled, RED, text)
}

fn wrap(enabled: bool, color: &str, text: String) -> String {
    if enabled {
        format!("{}{}{}", color, text, RESET)
    } else {
        text
    }
}