pub mod stream {

    use prelude::*;
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;

    pub struct ClientBuilder {
        token: Option<String>,
        account_id: Option<String>,
        webhook_port: Option<u16>,
    }

    struct Credentials {
        token: String,
        account_id: String,
    }

    impl ClientBuilder {
        pub fn new() -> Self {
            Self{
                token: None,
                account_id: None,
                webhook_port: None,
            }
        }

        pub fn token(mut self, token: String) -> Self {
            self.token = Some(token);
            self
        }

        pub fn account_id(mut self, account_id: String) -> Self {
            self.account_id = Some(account_id);
            self
        }

        pub async fn build(self) -> Result<Client> {
            match (self.token, self.account_id) {
                (Some(token), Some(account_id)) => Ok(Client::new(Credentials{ token, account_id }, 0).await),
                (_, _) => Err(anyhow!("Failed to build client")),
            }
        }
    }

    pub struct Client {
        credentials: Credentials,
    }

    impl Client {
        async fn new(credentials: Credentials, _webhook_port: u16) -> Self {
            Client{
                credentials
            }
        }

        pub async fn upload_video(&self, video: VideoRequest) -> Result<Video> {
            let mut video: Video = self.cloudflare_post(format!("https://api.cloudflare.com/client/v4/accounts/{}/stream/copy", self.credentials.account_id), &video).await?;
            let uid = video.uid.clone();

            while !video.ready_to_stream {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                video = self.cloudflare_get(format!("https://api.cloudflare.com/client/v4/accounts/{}/stream/{}", self.credentials.account_id, uid)).await?;
            }

            Ok(video)
        }

        async fn cloudflare_post<Req, Res>(&self, url: String, req: &Req) -> Result<Res>
            where
                Req: serde::Serialize + ?Sized,
                Res: serde::de::DeserializeOwned,
        {
            let client = reqwest::Client::new();
            let res = client.post(&url)
                .json(req)
                .header("Authorization", format!("Bearer: {}", self.credentials.token))
                .send()
                .await?;

            let result: CloudflareResponse<Res> = res.json::<CloudflareResponse<Res>>().await?;
            if !result.success {
                return Err(anyhow!("Cloudflare request failed: {:?}", result.errors))
            }

            match result.result {
                None => Err(anyhow!("Cloudflare request failed")),
                Some(r) => Ok(r)
            }
        }

        async fn cloudflare_get<Res>(&self, url: String) -> Result<Res>
            where
                Res: serde::de::DeserializeOwned,
        {
            let client = reqwest::Client::new();
            let result: CloudflareResponse<Res> = client.get(&url)
                .header("Authorization", format!("Bearer: {}", self.credentials.token))
                .send()
                .await?
                .json()
                .await?;

            if !result.success {
                return Err(anyhow!("Cloudflare request failed: {:?}", result.errors))
            }

            match result.result {
                None => Err(anyhow!("Cloudflare request failed")),
                Some(r) => Ok(r)
            }
        }
    }

    #[derive(serde::Serialize)]
    pub struct VideoRequest {
        pub url: String,
        pub meta: VideoMeta,
    }

    #[derive(serde::Serialize)]
    pub struct VideoMeta {
        pub name: String,
    }

    #[derive(serde::Deserialize)]
    pub struct Video {
        pub uid: String,
        pub preview: String,

        #[serde(rename = "readyToStream")]
        ready_to_stream: bool,
    }

    #[derive(serde::Deserialize)]
    pub struct CloudflareResponse<T> {
        #[serde(bound(deserialize = "Option<T>: serde::Deserialize<'de>"))]
        result: Option<T>,
        success: bool,
        errors: Vec<String>,
        messages: Vec<String>,
    }

}
