use std::{fs};
use std::path::Path;

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use chrono::{self, Local};
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};
use tokio_cron_scheduler::{Job, JobScheduler};
use log::{info, error};

extern crate log;

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


#[derive(Serialize, Deserialize)]
struct OptConfig {
    // 当期启动的端口号
    listener: String,
    // 缓存失效时间
    cache_expiration_time: i64,
    // 缓存的执行表达式
    cache_cron: String,
}

#[derive(Serialize, Deserialize)]
struct CacheInfo {
    // 文件创建时间
    create_time: i64,
    // 文件最后一次访问时间
    last_visit_time: i64,
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    if !Path::new(".opt.json").exists() {
        let default_config = OptConfig {
            listener: String::from("0.0.0.0:8011"),
            cache_expiration_time: 20160,
            cache_cron: String::from("0 */1 * * * *"),
        };
        fs::write(".opt.json", serde_json::to_string_pretty(&default_config).unwrap()).unwrap();
    }

    let config_str = fs::read_to_string(".opt.json")?;

    let opt_config: OptConfig = serde_json::from_str(config_str.as_str())?;

    info!("start up tcp://{}", opt_config.listener);

    let mut sched = JobScheduler::new();

    sched.add(Job::new(opt_config.cache_cron.as_str(), |_uuid, _l| {
        let config_str = fs::read_to_string(".opt.json").unwrap();
        let opt_config: OptConfig = serde_json::from_str(config_str.as_str()).unwrap();

        for entry in fs::read_dir(".cache").unwrap(){ 
            let entry = entry.unwrap();
            let file_name = entry.file_name();
            let last = entry.path();
            let real_path = last.to_str().unwrap();

            let extension = Path::new(&file_name).extension();
            if extension.expect("").eq("json") {
                let info_str = fs::read_to_string(&real_path).unwrap();
                let info: CacheInfo = serde_json::from_str(info_str.as_str()).unwrap();

                // 缓存过期，删除缓存的数据信息
                if Local::now().timestamp_millis() - info.last_visit_time >= opt_config.cache_expiration_time * 60 * 1000 {
                    let json_file = file_name.to_str().unwrap();
                    let split_file: Vec<&str> = json_file.split(".").collect();
                    let accept_file=split_file[0];
                    let pdf_file = format!("{}.pdf", accept_file);

                    if Path::new(json_file).exists() {
                        fs::remove_file(json_file).unwrap();
                    }
                    if Path::new(accept_file).exists() {
                        fs::remove_file(accept_file).unwrap();
                    }
                    if Path::new(&pdf_file).exists() {
                        fs::remove_file(&pdf_file).unwrap();
                    }
                }
            }
        }
    }).unwrap()).unwrap();

    sched.start();

    let listener = TcpListener::bind(opt_config.listener).await?;
    if !Path::new(".cache").exists() {
        fs::create_dir(".cache")?;
    }

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {

            let mut file_type_buffer = [0;4];

            let mut length_buffer = [0;4];
            socket.read(&mut file_type_buffer).await.unwrap();
            socket.read(&mut length_buffer).await.unwrap();
            let accept_length = as_u32_le(&length_buffer);
    
            let mut count_length: usize = 0;

            let mut content = vec![0; accept_length as usize];
            
            info!("receive file bytes [{}] startup.", accept_length);
            while count_length < accept_length as usize {

                let n = match socket.read(&mut content[count_length..]).await {
                    Ok(n) if n == 0 => n,
                    Ok(n) => {
                        n
                    },
                    Err(e) => {
                        error!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
                count_length += n;
                info!("receive -> {}/{}", count_length, accept_length);
            }
            info!("receive file bytes [{}] finish.", accept_length);

            let mut hasher = Sha256::new();
            hasher.update(&content);
            let result = hasher.finalize();

            if content.len() > 1 {
                let file_type = as_u32_le(&file_type_buffer);
                
                let id = format!("{:X}", result);
                let file_path = format!(".cache/{}", id);
                
                if !Path::new(&file_path).exists() {
                    fs::write(&file_path, &content).unwrap();
                    let now = Local::now().timestamp_millis();
                    let info = CacheInfo {
                        create_time: now,
                        last_visit_time: now,
                    };
                    fs::write(format!("{}.json", file_path), serde_json::to_string(&info).unwrap()).unwrap();
                } else {
                    let now = Local::now().timestamp_millis();
                    let info_file_path = format!("{}.json", file_path);

                    let info_str = fs::read_to_string(&info_file_path).unwrap();
                    let mut info: CacheInfo = serde_json::from_str(info_str.as_str()).unwrap();
                    info.last_visit_time = now;

                    fs::write(&info_file_path,serde_json::to_string(&info).unwrap()).unwrap(); 
                }
            
                let ms_file_type;
                if file_type  == 0 {
                    ms_file_type = MSFileType::WORD
                } else if file_type == 1 {
                    ms_file_type = MSFileType::EXCEL
                } else if file_type == 2 {
                    ms_file_type = MSFileType::PPT
                } else {
                    socket.shutdown().await.unwrap();
                    return;
                }

                let pdf_path = format!("{}.pdf", &file_path);
                if !Path::new(&pdf_path).exists() {
                    vbs::ms_export::ms_export_pdf(
                        file_path.as_str(),
                        &format!(".cache/{}.pdf", id),
                        ms_file_type,
                    ).await.unwrap();
                }

                if !Path::new(&pdf_path).exists() {
                    error!(" {}", "Pdf file generation failed.");
                    return;
                }

                let pdf_content = fs::read(&pdf_path).unwrap();

                socket.write_all(&as_u32_array_u8(pdf_content.len())).await.unwrap();

                if let Err(e) = socket.write_all(&pdf_content[..]).await {
                    error!("failed to write to socket; err = {:?}", e);
                    return;
                };
                socket.flush().await.unwrap();
                info!("RECEIVE -> Sha: {} - Size: {:.4} Kb",id, &content.len() / 1000);
            } else {
                error!("Incorrect data format.");
            }
            socket.shutdown().await.unwrap();
        });
    }
}
