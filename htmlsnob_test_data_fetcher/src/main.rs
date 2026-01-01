use reqwest::{header, Client};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
struct SearchResponse {
    items: Vec<Item>,
    total_count: usize,
}

#[derive(Debug, Deserialize)]
struct Item {
    repository: Repository,
}

#[derive(Debug, Deserialize)]
struct Repository {
    full_name: String,
}

const MAX_REPO_SIZE_KB: usize = 50000; // 50MB
const GITHUB_API_URL: &str = "https://api.github.com";

struct Language<'a> {
    query: &'a str,
    name: &'a str,
    extensions: Vec<&'a str>,
}

#[tokio::main]
async fn main() {
    let languages = [
        Language {
            query: "extension:html.jinja2",
            name: "Jinja2",
            extensions: vec!["jinja", "j2", "jinja2"],
        },
        Language {
            query: "extension:html.liquid",
            name: "Liquid",
            extensions: vec!["liquid"],
        },
        Language {
            query: "extension:html.twig",
            name: "Twig",
            extensions: vec!["twig"],
        },
        Language {
            query: "extension:html.eex",
            name: "Eex",
            extensions: vec!["eex", "leex"],
        },
        Language {
            query: "extension:html.erb",
            name: "Erb",
            extensions: vec!["erb"],
        },
        Language {
            query: "extension:html.ejs",
            name: "Ejs",
            extensions: vec!["ejs"],
        },
        Language {
            query: "extension:html.tmpl",
            name: "Go Templates",
            extensions: vec!["tmpl"],
        },
        Language {
            query: "extension:html.hbs",
            name: "Handlebars",
            extensions: vec!["hbs", "hbs.html", "hbs.handlebars"],
        },
        Language {
            query: "extension:html.mustache",
            name: "Mustache",
            extensions: vec!["html"],
        },
    ];

    for lang in languages.iter() {
        process_language(lang).await;
    }
}

async fn process_language<'a>(language: &Language<'a>) {
    // Get GitHub token from environment
    let github_token =
        dotenvy::var("GITHUB_TOKEN").expect("GITHUB_TOKEN environment variable must be set");

    // Create HTTP client with GitHub API headers
    let client = create_github_client(&github_token);

    let output_base_dir_name = format!("output/{}", language.name);
    let output_base_dir = Path::new(&output_base_dir_name);
    fs::create_dir_all(output_base_dir).unwrap();

    // Search for files
    let search_results = search_files(&client, language).await.await;
    println!(
        "Found {} repositories with {} files",
        &language.name, search_results.total_count
    );

    // Process each repository
    for item in search_results.items {
        let repo_name = item.repository.full_name.clone();
        println!("Processing repository: {}", repo_name);

        // Check repository size by calling API, and skip if too large
        let repo_info_url = format!("{}/repos/{}", GITHUB_API_URL, repo_name);

        let response = client.get(&repo_info_url).send().await.unwrap();

        let body = response.text().await.unwrap();

        let json: Value = serde_json::from_str(&body).unwrap();

        if let Some(size) = json["size"].as_u64() {
            // GitHub API returns size in KB
            if size > MAX_REPO_SIZE_KB as u64 {
                println!(
                    "Skipping {} - too large ({} KB > {} KB limit)",
                    repo_name, size, MAX_REPO_SIZE_KB
                );
                continue;
            }
            println!("Repository size: {} KB", size);
        } else {
            println!(
                "Couldn't determine size for {}, proceeding anyway",
                repo_name
            );
        }

        // Extract repo info
        let repo_url = format!("https://github.com/{}.git", repo_name);

        // Create temporary directory for cloning
        let temp_dir = format!("temp_{}", repo_name.replace('/', "_"));
        let _ = fs::remove_dir_all(&temp_dir); // Clean up any existing directory
        fs::create_dir_all(&temp_dir).unwrap();

        println!("Cloning {} to temporary directory...", repo_name);

        let clone_output = Command::new("git")
            .args(["clone", "--depth=1", &repo_url, &temp_dir])
            .output()
            .unwrap();

        if !clone_output.status.success() {
            println!("Failed to clone repository {}", repo_name);
            println!("Error: {}", String::from_utf8_lossy(&clone_output.stderr));
            let _ = fs::remove_dir_all(&temp_dir);
            continue;
        }

        // Find and copy files
        println!("Extracting files from {}", repo_name);
        extract_files(&temp_dir, &output_base_dir_name, &language.extensions);

        // Clean up temporary directory
        let _ = fs::remove_dir_all(&temp_dir);
    }

    println!("Done!")
}

fn create_github_client(token: &str) -> Client {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/vnd.github.v3+json"),
    );
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("test-data-extractor"),
    );
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("token {}", token)).unwrap(),
    );

    Client::builder().default_headers(headers).build().unwrap()
}

async fn search_files(
    client: &Client,
    language: &Language<'_>,
) -> Pin<Box<dyn Future<Output = SearchResponse> + Send>> {
    let per_page = 300;
    let url = format!(
        "{}/search/code?q={}&per_page={}",
        GITHUB_API_URL, language.query, per_page
    );

    let response = client.get(&url).send().await.unwrap();
    let body = response.text().await.unwrap();

    println!("Response: {}", body);

    let json: Value = serde_json::from_str(&body).unwrap();

    // Check for errors
    if let Some(message) = json.get("message") {
        if let Some(status) = json.get("status") {
            if status == "403" {
                println!("Error: {}", message);
                sleep(Duration::from_secs(30)).await;
                return Box::pin(search_files(client, language)).await; // Boxed recursive call
            } else {
                println!("Error: {}", message);
            }
        } else {
            println!("Error: {}", message);
        }
    }

    let search_response: SearchResponse = serde_json::from_str(&body).unwrap();
    Box::pin(async move { search_response }) // Wrap final return value in Box::pin
}

fn extract_files(source_dir: &str, target_dir: &str, extensions: &Vec<&str>) {
    let mut count = 0;

    // Use a simple function to walk directories recursively
    fn walk_dir(
        dir: &Path,
        source_base: &Path,
        target_dir: &str,
        count: &mut i32,
        extensions: &Vec<&str>,
    ) {
        if dir.is_dir() {
            let entries = fs::read_dir(dir).unwrap();
            for entry in entries {
                let path = entry.unwrap().path();

                if path.is_dir() {
                    walk_dir(&path, source_base, target_dir, count, extensions);
                } else if path.is_file() {
                    // Check if it's a file
                    let captured = extensions.iter().any(|ext| {
                        path.to_string_lossy()
                            .ends_with(format!(".html.{}", ext).as_str())
                    });

                    if captured {
                        // Get the relative path from source base
                        let rel_path = path
                            .strip_prefix(source_base)
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .replace('\\', "/");

                        // Create a unique filename by flattening the path
                        let sanitized_name = format!(
                            "{}_{}",
                            source_base.to_str().unwrap(),
                            rel_path.replace('/', "_")
                        );
                        let sanitized_name = truncate(&sanitized_name, 150);

                        // Copy the file to the target directory
                        let target_path = format!("{}/{}", target_dir, sanitized_name);
                        println!("  To copy: {} -> {}", rel_path, target_path);
                        fs::copy(&path, &target_path).unwrap();

                        *count += 1;
                    }
                }
            }
        }
    }

    let source_path = Path::new(source_dir);
    walk_dir(source_path, source_path, target_dir, &mut count, extensions);

    println!("Extracted {} files", count);
}

fn truncate(s: &str, len: usize) -> String {
    if s.len() <= len {
        return s.to_string();
    }

    let unix_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos()
        .to_string();

    let start = s.len().saturating_sub(len);
    let f = s
        .char_indices()
        .nth(start)
        .map(|(idx, _)| &s[idx..])
        .unwrap_or("")
        .to_string();

    format!("{}-{}", f, unix_time)
}
