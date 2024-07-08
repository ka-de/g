use redis::AsyncCommands;

#[tokio::main]
async fn main() -> redis::RedisResult<()> {
    // Open a connection to the Redis server.
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_multiplexed_async_connection().await?;

    // Get the hash from Redis.
    let repos: std::collections::HashMap<String, String> = con.hgetall("github_repos").await?;

    // Print the keys and values.
    for (key, value) in repos.iter() {
        println!("{key}: {value}");
    }

    Ok(())
}
