use octocrab::Octocrab;
use std::env;
use std::fs;
use std::path::Path;
use getopts::Options;

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let token = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");

    let octocrab = Octocrab::builder().personal_token(token).build()?;

    // Get the command line arguments
    let args: Vec<String> = env::args().collect();

    // Define the options
    let mut opts = Options::new();
    opts.optflag("", "public", "make the gist public");
    opts.optflag("", "private", "make the gist private");

    // Parse the options
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!("{}", f.to_string()),
    };

    // The first argument is the file path, the second is the description
    let file_path = if !matches.free.is_empty() {
        &matches.free[0]
    } else {
        panic!("No file path provided");
    };
    let description = if matches.free.len() > 1 { &matches.free[1] } else { "" };

    // Determine if the gist should be public or private
    let is_public = if matches.opt_present("private") {
        false
    } else {
        // Default to public if no option is provided
        true
    };

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
        .public(is_public)
        .send().await?;
    println!("Done, created: {url}", url = gist.html_url);
    Ok(())
}
