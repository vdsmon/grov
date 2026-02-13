use std::io::{self, BufRead, Write};

use console::style;

pub fn prompt(label: &str, default: Option<&str>, reader: &mut impl BufRead) -> io::Result<String> {
    let prompt_marker = style("?").cyan().bold();
    let label_styled = style(label).bold();
    match default {
        Some(d) => eprint!(
            "{prompt_marker} {label_styled} {}: ",
            style(format!("[{d}]")).dim()
        ),
        None => eprint!("{prompt_marker} {label_styled}: "),
    }
    io::stderr().flush()?;
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line.trim().to_string())
}
