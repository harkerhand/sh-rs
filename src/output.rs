use colored::{Color, Colorize};

pub fn print_with_color(message: &str, color: Color) {
    println!("{}", message.color(color));
}

#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {
        crate::output::print_with_color(&format!($($arg)*), colored::Color::Red)
    };
}

#[macro_export]
macro_rules! println_error {
    ($($arg:tt)*) => {
        crate::output::print_with_color(&format!($($arg)*), colored::Color::Red)
    };
}
