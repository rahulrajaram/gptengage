//! Output formatting utilities

pub struct Formatter;

impl Formatter {
    pub fn header(text: &str) {
        println!("{}", text);
    }

    pub fn subheader(text: &str) {
        println!("{}", text);
    }

    pub fn success(text: &str) {
        println!("✓ {}", text);
    }

    pub fn error(text: &str) {
        println!("✗ {}", text);
    }

    pub fn info(text: &str) {
        println!("  {}", text);
    }

    pub fn table_header(headers: &[&str]) {
        println!("{}", headers.join(" | "));
    }

    pub fn table_row(cells: &[&str]) {
        println!("{}", cells.join(" | "));
    }
}
