mod cli;

use crate::cli::CLI;
use reqwest::Client;
use std::env;
use std::path::Path;
use std::sync::Arc;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
    sync::Semaphore,
    task,
    time::{sleep, Duration},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: CLI = CLI::new(env::args().collect());

    let ua = arguments
        .get::<String>("ua")
        .unwrap_or("Mozilla/5.0 (Windows NT x.y; rv:10.0) Gecko/20100101 Firefox/10.0".to_string());

    let follow = arguments.get::<bool>("follow").unwrap_or(false);
    let threads = arguments.get::<usize>("workers").unwrap_or(10);
    let ignore_cmd = arguments
        .get::<String>("ignore")
        .unwrap_or(String::from("0"));

    let ignore: Vec<i32> = ignore_cmd
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let delay = arguments.get::<usize>("delay").unwrap_or(0);

    println!("code,host,url,title,hash,bytes");

    if let Some(dict) = arguments.get::<String>("dict") {
        if let Some(host) = arguments.get::<String>("host") {
            if Path::new(dict.as_str()).exists() {
                let file = File::open(dict.as_str()).await?;
                let reader = BufReader::new(file);

                let mut client_builder = Client::builder();

                if !ua.is_empty() {
                    client_builder = client_builder.user_agent(ua);
                }
                if follow {
                    client_builder = client_builder.redirect(reqwest::redirect::Policy::default());
                } else {
                    client_builder = client_builder.redirect(reqwest::redirect::Policy::none());
                }

                let client = Arc::new(client_builder.build()?);
                let semaphore = Arc::new(Semaphore::new(threads)); // Shared Semaphore
                let mut lines = reader.lines();

                let chunk_size = 100;
                let mut chunk = Vec::with_capacity(chunk_size);
                while let Some(line) = lines.next_line().await? {
                    if !line.starts_with("#") && !line.is_empty() {
                        chunk.push(line);
                        if chunk.len() == chunk_size {
                            process_chunk(
                                &host,
                                Arc::clone(&client),
                                chunk.clone(),
                                Arc::clone(&semaphore),
                                ignore.clone(),
                                delay,
                            )
                            .await;
                            chunk.clear();
                        }
                    }
                }
                if !chunk.is_empty() {
                    process_chunk(
                        &host,
                        Arc::clone(&client),
                        chunk,
                        Arc::clone(&semaphore),
                        ignore.clone(),
                        delay,
                    )
                    .await;
                }
            }
        }
    }
    Ok(())
}

fn get_title(html: String) -> Option<String> {
    if let Some(start) = html.find("<title>") {
        if let Some(end) = html[start..].find("</title>") {
            return Some(html[start + 7..start + end].to_string());
        }
    }
    None
}

async fn request(
    client: Arc<Client>,
    host: &String,
    url: &String,
    ignore: Vec<i32>,
) -> Result<String, reqwest::Error> {
    let response = client.get(url).send().await?;

    let status_code = response.status().as_u16() as i32;
    let content = response.text().await?;
    let title = get_title(content.clone())
        .unwrap_or("NO TITLE".to_string())
        .trim()
        .replace("\t", "")
        .replace("\r", "")
        .replace("\n", "");

    let content_hash = md5::compute(content.clone().into_bytes());
    let content_size = content.len();

    if !ignore.contains(&status_code) {
        return Ok(format!(
            "{},{},{},{},{:x},{}",
            status_code, host, url, title, content_hash, content_size
        ));
    }
    Ok(String::from(""))
}

async fn process_chunk(
    host: &str,
    client: Arc<Client>,
    chunk: Vec<String>,
    semaphore: Arc<Semaphore>, // Pass semaphore here
    ignore: Vec<i32>,
    delay: usize,
) {
    let mut tasks = vec![];
    for line in chunk {
        let semaphore = Arc::clone(&semaphore);
        let url = host.replace("[FUZZ]", line.as_str());
        let host = host.replace("[FUZZ]", "");
        let client = Arc::clone(&client);
        let ignore = ignore.clone();

        let task = task::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            if delay > 0 {
                sleep(Duration::from_secs(delay as u64)).await;
            }

            if !url.is_empty() {
                if let Ok(data) = request(client, &host, &url, ignore).await {
                    if data.chars().count() > 10 {
                        println!("{}", data);
                    }
                }
            }
            drop(_permit); // Explicitly release permit
        });
        tasks.push(task);
    }

    for task in tasks {
        let _ = task.await;
    }
}
