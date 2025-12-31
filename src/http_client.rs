use super::errors::Result;
use async_trait::async_trait;

pub struct Response {
    pub status: u16,
    pub body: String,
}

#[async_trait]
pub trait HttpClient {
    async fn get(&self, url: &str, api_key: &str) -> Result<Response>;
}

#[cfg(feature = "reqwest-client")]
#[async_trait]
impl HttpClient for reqwest::Client {
    async fn get(&self, url: &str, api_key: &str) -> Result<Response> {
        Response::from(
            self.get(url)
                .header("Authorization", format!("apikey {api_key}"))
                .send()
                .await?,
        )
        .await
    }
}

#[cfg(feature = "reqwest-client")]
impl Response {
    async fn from(r: reqwest::Response) -> Result<Self> {
        Ok(Self {
            status: r.status().as_u16(),
            body: r.text().await?,
        })
    }
}

#[cfg(feature = "surf-client")]
#[async_trait]
impl HttpClient for surf::Client {
    async fn get(&self, url: &str, api_key: &str) -> Result<Response> {
        Response::from(
            self.get(url)
                .header("Authorization", format!("apikey {}", api_key))
                .send()
                .await?,
        )
        .await
    }
}

#[cfg(feature = "surf-client")]
impl Response {
    async fn from(r: surf::Response) -> Result<Self> {
        let mut r = r;
        Ok(Self {
            status: r.status().into(),
            body: r.body_string().await?,
        })
    }
}

#[cfg(feature = "wreq-client")]
#[async_trait]
impl HttpClient for wreq::Client {
    async fn get(&self, url: &str, api_key: &str) -> Result<Response> {
        Response::from(
            self.get(url)
                .header("Authorization", format!("apikey {}", api_key))
                .send()
                .await?,
        )
        .await
    }
}

#[cfg(feature = "wreq-client")]
impl Response {
    async fn from(r: wreq::Response) -> Result<Self> {
        Ok(Self {
            status: r.status().into(),
            body: r.text().await?,
        })
    }
}
