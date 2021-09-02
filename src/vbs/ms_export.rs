use std::{io::Error, process::{Command, Output}};

pub async fn ms_word_export_pdf(input: &str, output: &str) -> Result<Output, Error> {
    Command::new("cscript").arg(".scripts/ns-word.vbs").arg(input).arg(output).output()
}
