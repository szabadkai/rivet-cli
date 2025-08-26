use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new(timeout: Duration, insecure: bool) -> Result<Self> {
        let mut builder = Client::builder()
            .timeout(timeout);
            
        if insecure {
            builder = builder.danger_accept_invalid_certs(true);
        }
        
        Ok(Self {
            client: builder.build()?,
        })
    }
    
    pub fn client(&self) -> &Client {
        &self.client
    }
}