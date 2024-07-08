use octocrab::Octocrab;
use std::env;
use std::fs;
use std::path::Path;

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let token = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");

    let octocrab = Octocrab::builder().personal_token(token).build()?;

    // Get the command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <file_path> <description>", args[0]);
        std::process::exit(1);
    }

    // The first argument is the file path, the second is the description
    let file_path = &args[1];
    let description = &args[2];

    // Strip the file path from the name
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(file_path);

    // Read the file content
    let content = fs::read_to_string(file_path).expect("Could not read file");

    println!("Creating a gist with the content of {} on your account", file_name);
    let gist = octocrab
        .gists()
        .create()
        .file(file_name, &content)
        // Optional Parameters
        .description(description)
        .public(false)
        .send().await?;
    println!("Done, created: {url}", url = gist.html_url);
    Ok(())
}
