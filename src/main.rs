use std::fs;
use std::path::Path;

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bytes::{BufMut, BytesMut};
use chrono;
use sha2::{Sha256, Digest};

mod vbs;

fn as_u32_le(array: &[u8]) -> u32 {
    ((array[0] as u32) <<  0) +
    ((array[1] as u32) <<  8) +
    ((array[2] as u32) << 16) +
    ((array[3] as u32) << 24)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    if !Path::new(".cache").exists() {
        fs::create_dir(".cache")?;
    }

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            let mut content = BytesMut::with_capacity(1024);

            loop {
                let n = match socket.read(&mut buffer).await {
                    Ok(n) if n == 0 => 0,
                    Ok(n) => {
                        n
                    },
                    Err(e) => {
                        eprintln!("ERRPR -> failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
                
                if n == 0 {
                    break;
                }
                content.put(&buffer[0..n]);
            }
            
            let mut hasher = Sha256::new();
            hasher.update(&content);
            let result = hasher.finalize();

            if content.len() > 1 {
                let file_type = as_u32_le(&content[..4]);
                
                let id = format!("{:X}", result);
                let file_path = format!(".cache/{}", id);
                
                if !Path::new(&file_path).exists() {
                    fs::write(&file_path, &content[4..]).unwrap();
                }
            
                match file_type {
                    0 => {
                        let pdf_path = format!("{}.pdf", &file_path);
                        if !Path::new(&pdf_path).exists() {
                            match vbs::ms_export::ms_word_export_pdf(file_path.as_str(), &format!(".cache/{}.pdf", id)).await {
                                Err(e) => {
                                    eprintln!("ERRPR -> failed to word export pdf; err = {:?}", e);
                                }
                                Ok(_) => return ,
                            };
                        }
    
                        let content = fs::read(&file_path).unwrap();
                        if let Err(e) = socket.write_all(&content[0..]).await {
                            eprintln!("ERROR -> failed to write to socket; err = {:?}", e);
                            return;
                        };
                    }
                    _ => {
                        eprintln!("ERROR -> Unknown file type [{}]", file_type);
                    }
                }

                println!("{:?} RECEIVE -> Sha: {} - Size: {:.4} Kb", chrono::offset::Local::now(),id, (&content.len() -1 ) / 1000);
            } else {
                println!("{:?} ERRPR -> Incorrect data format [ \n {:X} \n]", chrono::offset::Local::now(), &content);
            }
        });
    }
}
