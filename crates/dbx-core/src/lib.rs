pub mod agent_manager;
pub mod agent_service;
pub mod ai;
pub mod connection;
pub mod connection_secrets;
pub mod database_capabilities;
pub mod database_export;
pub mod db;
pub mod external;
pub mod history;
pub mod models;
pub mod mongo_ops;
pub mod plugins;
pub mod query;
pub mod query_cancel;
pub mod redis_ops;
pub mod saved_sql;
pub mod schema;
pub mod sql;
pub mod storage;
pub mod table_import;
pub mod transfer;
pub mod types;
pub mod update;

pub const R2_CDN_BASE: &str = "https://dl.dbxio.com/";

pub fn download_candidate_urls(github_url: &str, r2_path: &str) -> Vec<String> {
    vec![format!("{R2_CDN_BASE}{r2_path}"), github_url.to_string()]
}

pub async fn race_download(
    client: &reqwest::Client,
    github_url: &str,
    r2_path: &str,
    user_agent: &str,
) -> Result<reqwest::Response, String> {
    use futures::future::select_ok;
    use std::pin::Pin;

    let urls = download_candidate_urls(github_url, r2_path);
    let mut futs: Vec<Pin<Box<dyn std::future::Future<Output = Result<reqwest::Response, String>> + Send>>> =
        Vec::with_capacity(urls.len());

    for url in urls {
        let client = client.clone();
        let ua = user_agent.to_string();
        futs.push(Box::pin(async move {
            client
                .get(&url)
                .header(reqwest::header::USER_AGENT, ua)
                .send()
                .await
                .and_then(|r| r.error_for_status())
                .map_err(|e| format!("{e}"))
        }) as Pin<Box<dyn std::future::Future<Output = Result<reqwest::Response, String>> + Send>>);
    }

    match select_ok(futs).await {
        Ok((resp, _)) => Ok(resp),
        Err(last_err) => Err(last_err),
    }
}

#[cfg(test)]
mod tests {
    use super::download_candidate_urls;

    #[test]
    fn download_candidates_exclude_third_party_github_proxy() {
        let urls = download_candidate_urls(
            "https://github.com/t8y2/dbx/releases/latest/download/latest.json",
            "releases/latest/latest.json",
        );

        assert_eq!(
            urls,
            vec![
                "https://dl.dbxio.com/releases/latest/latest.json",
                "https://github.com/t8y2/dbx/releases/latest/download/latest.json",
            ]
        );
    }
}
