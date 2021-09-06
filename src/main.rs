use std::convert::TryInto;
use std::fs;
use std::path::Path;

use bytes::{BufMut, BytesMut};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use chrono;
use sha2::{Sha256, Digest};

use crate::vbs::ms_export::MSFileType;

mod vbs;

fn as_u32_le(array: &[u8]) -> u32 {
    ((array[0] as u32) <<  0) +
    ((array[1] as u32) <<  8) +
    ((array[2] as u32) << 16) +
    ((array[3] as u32) << 24)
}


fn as_u32_array_u8(x:usize) -> [u8;4] {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4]
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ipaddr = "0.0.0.0:8011";

    let listener = TcpListener::bind("0.0.0.0:8011").await?;

    println!("=============================================");
    println!("|     -> start up  TCP://{} <-    |", ipaddr);
    println!("=============================================");

    if !Path::new(".cache").exists() {
        fs::create_dir(".cache")?;
    }

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buffer = [0; 10240];
            let mut content = BytesMut::with_capacity(1024);
  
            let mut length_buffer = [0;4];
            let mut file_type_buffer = [0;4];

            socket.read(&mut file_type_buffer).await.unwrap();
            socket.read(&mut length_buffer).await.unwrap();

            let accept_length = as_u32_le(&length_buffer);

            let mut count_length = 0;

            loop {

                let n = match socket.read(&mut buffer).await {
                    Ok(n) if n == 0 => n,
                    Ok(n) => {
                        n
                    },
                    Err(e) => {
                        eprintln!("ERRPR -> failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
                
                count_length += n;

                if accept_length == count_length.try_into().unwrap() {
                    break;
                }
                content.put(&buffer[0..n]);
            }


            let mut hasher = Sha256::new();
            hasher.update(&content);
            let result = hasher.finalize();

            if content.len() > 1 {
                let file_type = as_u32_le(&file_type_buffer);
                
                let id = format!("{:X}", result);
                let file_path = format!(".cache/{}", id);
                
                if !Path::new(&file_path).exists() {
                    fs::write(&file_path, &content).unwrap();
                }
            
                let ms_file_type = match file_type {
                    0 => MSFileType::WORD,
                    1 => MSFileType::EXCEL,
                    2 => MSFileType::PPT,
                    _ => MSFileType::WORD,
                };

                let pdf_path = format!("{}.pdf", &file_path);
                if !Path::new(&pdf_path).exists() {
                   let output = vbs::ms_export::ms_export_pdf(
                        file_path.as_str(),
                        &format!(".cache/{}.pdf", id),
                        ms_file_type,
                    ).await.unwrap();

                    let output= String::from_utf8(output.stdout).expect("failed to execute.");
                    println!("ERRPR -> {}", output)
                }

                if !Path::new(&pdf_path).exists() {
                    println!("ERRPR -> {}", "Pdf file generation failed.");
                    return;
                }

                let pdf_content = fs::read(&pdf_path).unwrap();

                socket.write_all(&as_u32_array_u8(pdf_content.len())).await.unwrap();

                if let Err(e) = socket.write_all(&pdf_content[0..]).await {
                    eprintln!("ERROR -> failed to write to socket; err = {:?}", e);
                    return;
                };
                socket.flush().await.unwrap();
                println!("{:?} RECEIVE -> Sha: {} - Size: {:.4} Kb", chrono::offset::Local::now(),id, (&content.len() -1 ) / 1000);
            } else {
                println!("{:?} ERRPR -> Incorrect data format.", chrono::offset::Local::now());
            }
        });
    }
}
