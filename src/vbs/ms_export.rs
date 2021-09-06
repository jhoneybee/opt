use std::{env, io::Error, process::{Command, Output}};

pub enum MSFileType {
    EXCEL,
    PPT,
    WORD,
}

pub async fn ms_export_pdf(input: &str, output: &str, file_type: MSFileType) -> Result<Output, Error> {

    let path = env::current_dir()?;

    let command_type = match file_type {
        MSFileType::EXCEL => "EXCEL",
        MSFileType::WORD => "WORD",
        MSFileType::PPT => "PPT",
    };

    let command = format!("cscript {}/.scripts/ms-call.vbs {} {} {}", path.display(), input, output, command_type);

    Command::new("cmd")
    .args(&["/C", command.as_str()])
    .output()
}
