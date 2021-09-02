use std::fs;
use std::path::Path;

use tokio::net::TcpListener;
use tokio::io::{ AsyncReadExt };
use bytes::{BufMut, BytesMut};
use chrono;
use sha2::{Sha256, Digest};

fn as_u32_le(array: &[u8]) -> u32 {
    ((array[0] as u32) <<  0) +
    ((array[1] as u32) <<  8) +
    ((array[2] as u32) << 16) +
    ((array[3] as u32) << 24)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("192.168.10.194:8080").await?;

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
                        eprintln!("failed to read from socket; err = {:?}", e);
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

                let id = format!("{:X}-{}", result,  as_u32_le(&content[..4]));
    
                let file_path = format!(".cache/{}", id);
    
                if !Path::new(&file_path).exists() {
                    fs::write(file_path, &content[4..]).unwrap();
                }
                println!("{:?} RECEIVE -> Sha: {} - Size: {:.4} Kb", chrono::offset::Local::now(),id, (&content.len() -1 ) / 1000);
            } else {
                println!("{:?} ERRPR -> Incorrect data format [ \n {:X} \n]", chrono::offset::Local::now(), &content);
            }
        });
    }
}
