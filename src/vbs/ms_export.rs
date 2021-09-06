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

    let real_input_path = format!("{}/{}", path.display(), input);
    let real_output_path = format!("{}/{}", path.display(), output);

    let command = format!("cscript {}/.scripts/ms-call.vbs {} {} {}", path.display(), real_input_path, real_output_path, command_type);

    println!("{}", command);

    Command::new("cmd")
    .args(&["/C", command.as_str()])
    .output()
}
