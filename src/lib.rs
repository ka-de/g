use octocrab::Octocrab;
use ::redis::AsyncCommands;
use ::redis::aio;
use std::error::Error;
use url::Url;

pub mod github {
    use super::*;

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
