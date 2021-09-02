use std::{ io::Error, process::{Command, Output}};

pub enum MSFileType {
    EXCEL,
    PPT,
    WORD,
}

pub async fn ms_export_pdf(input: &str, output: &str, file_type: MSFileType) -> Result<Output, Error> {

    let command_type = match file_type {
        MSFileType::EXCEL => "EXCEL",
        MSFileType::WORD => "WORD",
        MSFileType::PPT => "PPT",
    };

    Command::new("cscript")
        .arg(".scripts/ms-call.vbs")
        .arg(input)
        .arg(output)
        .arg(command_type)
    .output()
}
