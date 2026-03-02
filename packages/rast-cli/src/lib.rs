mod commands;
mod io;
mod output;

pub use commands::{run::run, scan::scan};
pub use output::types::OutputFormat;

#[cfg(test)]
mod tests;
