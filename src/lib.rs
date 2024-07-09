use octocrab::Octocrab;
use ::redis::aio;
use ::redis::AsyncCommands;
use std::error::Error;
use url::Url;

pub mod github {
    use super::*;

    pub async fn get_repo_stats(
        octocrab: &Octocrab,
        owner: &str,
        repo: &str
    ) -> Result<(String, u32, u8), Box<dyn Error>> {
        let repo_info = octocrab.repos(owner, repo).get().await?;
        let repo_metrics = octocrab.repos(owner, repo).get_community_profile_metrics().await?;

        let full_name = repo_info.full_name.unwrap_or_else(|| format!("{}/{}", owner, repo));
        let stars = repo_info.stargazers_count.unwrap_or(0);
        let health_percentage = repo_metrics.health_percentage;

        Ok((full_name, stars, health_percentage.try_into().unwrap()))
    }

    // Add this to the existing redis module in lib.rs
    pub async fn store_repos(
        con: &mut ::redis::aio::MultiplexedConnection,
        repos: &[(String, String)]
    ) -> Result<(), Box<dyn Error>> {
        for (name, url) in repos {
            let _: () = con.hset("github_repos", name, url).await?;
        }
        Ok(())
    }

    pub async fn create_gist(
        octocrab: &Octocrab,
        file_name: &str,
        content: &str,
        description: &str,
        is_public: bool
    ) -> Result<String, Box<dyn Error>> {
        let gist = octocrab
            .gists()
            .create()
            .file(file_name, content)
            .description(description)
            .public(is_public)
            .send().await?;

        Ok(gist.html_url.to_string())
    }

    pub async fn list_repos(octocrab: &Octocrab) -> Result<Vec<(String, String)>, Box<dyn Error>> {
        let mut repos = Vec::new();
        let mut page = octocrab
            .current()
            .list_repos_for_authenticated_user()
            .per_page(100)
            .send().await?;

        loop {
            for repo in &page.items {
                let name = repo.name.clone();
                let url = repo.html_url
                    .as_ref()
                    .unwrap_or(&Url::parse("https://github.com").unwrap())
                    .to_string();
                repos.push((name, url));
            }

            if let Some(next_page) = octocrab.get_page(&page.next).await? {
                page = next_page;
            } else {
                break;
            }
        }

        Ok(repos)
    }
}

pub mod redis {
    use super::*;

    pub async fn store_repos(
        con: &mut redis::aio::MultiplexedConnection,
        repos: &[(String, String)]
    ) -> Result<(), Box<dyn Error>> {
        for (name, url) in repos {
            let _: () = con.hset("github_repos", name, url).await?;
        }
        Ok(())
    }
}

pub fn build_octocrab(token: &str) -> Result<Octocrab, Box<dyn Error>> {
    Ok(Octocrab::builder().personal_token(token.to_string()).build()?)
}
