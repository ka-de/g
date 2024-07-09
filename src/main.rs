use g::{ build_octocrab, github, redis };
use getopts::Options;
use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;
use ::redis::Client;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("Error: {e}");
        exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("", "public", "make the gist public");
    opts.optflag("", "private", "make the gist private");
    opts.optflag("h", "help", "print this help menu");

    let matches = opts.parse(&args[1..])?;

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Ok(());
    }

    let token = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
    let octocrab = build_octocrab(&token)?;

    match matches.free.get(0).map(String::as_str) {
        Some("store-repos") => {
            let repos = github::list_repos(&octocrab).await?;
            let client = Client::open("redis://127.0.0.1/")?;
            let mut con = client.get_multiplexed_async_connection().await?;
            redis::store_repos(&mut con, &repos).await?;
            println!("Repositories stored in Redis");
        }
        Some("repo-stats") => {
            if matches.free.len() < 3 {
                eprintln!("Not enough arguments for repo-stats command");
                print_usage(&program, opts);
                exit(1);
            }
            let owner = &matches.free[1];
            let repo = &matches.free[2];
            let (full_name, stars, health_percentage) = github::get_repo_stats(
                &octocrab,
                owner,
                repo
            ).await?;
            println!("{full_name} has {stars} stars and {health_percentage}% health percentage");
        }
        Some("gist") => {
            if matches.free.len() < 3 {
                eprintln!("Not enough arguments for gist command");
                print_usage(&program, opts);
                exit(1);
            }

            let file_path = &matches.free[1];
            let description = &matches.free[2];
            let is_public = !matches.opt_present("private");

            let file_name = Path::new(file_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(file_path);

            let content = fs::read_to_string(file_path)?;

            let url = github::create_gist(
                &octocrab,
                file_name,
                &content,
                description,
                is_public
            ).await?;
            println!("Gist created: {url}");
        }
        Some("list-repos") => {
            let repos = github::list_repos(&octocrab).await?;
            for (name, url) in repos {
                println!("{name}: {url}");
            }
        }
        _ => {
            print_usage(&program, opts);
            exit(1);
        }
    }

    Ok(())
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] COMMAND", program);
    print!("{}", opts.usage(&brief));
    println!("\nCommands:");
    println!("  gist <file_path> <description>  Create a gist");
    println!("  list-repos                      List user repositories");
    println!("  store-repos                     Store user repositories in Redis");
    println!("  repo-stats <owner> <repo>       Get statistics for a specific repository");
}
