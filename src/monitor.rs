use crate::country::Country;
use crate::model::FPSProduct;
use crate::model::Variant;
use crate::Error;
use log::info;
use log::warn;
use reqwest::header::ACCEPT_LANGUAGE;
use reqwest::header::{HeaderMap, HeaderValue, HOST, USER_AGENT};
use reqwest::Client;
use reqwest::Url;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use strum::AsStaticRef;
use tokio::sync::broadcast::Sender;

pub struct Monitor {
    product: String,
    client: Client,
    base_url: Url,
    sender: Sender<(i64, Vec<Variant>)>,
}

impl Monitor {
    pub fn new(
        product: String,
        country: &Country,
        sender: Sender<(i64, Vec<Variant>)>,
    ) -> Result<Monitor, Error> {
        let base_url = Url::parse(country.fps_base_url())?;
        let mut headers = HeaderMap::new();
        headers.insert(HOST, HeaderValue::from_static("www.emiliopucci.com:443"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0",
            ),
        );
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static(country.accept_language()),
        );
        headers.insert("FF-Country", HeaderValue::from_static(country.as_static()));
        headers.insert(
            "FF-Currency",
            HeaderValue::from_static(country.fps_currency()),
        );

        let client = Client::builder()
            .use_rustls_tls()
            .gzip(true)
            .default_headers(headers)
            .cookie_store(true)
            .user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 14_4_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1")
            .build()?;

        Ok(Monitor {
            product,
            client,
            base_url,
            sender,
        })
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        loop {
            match self.fetch_product().await {
                Ok(product) => {
                    let variants = product
                        .result
                        .variants
                        .iter()
                        .cloned()
                        .filter(|variant| variant.quantity > 0)
                        .collect::<Vec<_>>();

                    if variants.len() > 0 {
                        info!(
                            "product={} message=\"variants loaded - {}\"",
                            &self.product,
                            variants.len()
                        );
                        if let Err(why) = self.sender.send((product.result.id, variants)) {
                            warn!("{}", why);
                        }
                    } else {
                        warn!("product={} message=\"no variants loaded\"", &self.product)
                    }
                }
                Err(why) => {
                    dbg!(why);
                }
            }

            let duration = Duration::from_millis(1000);

            tokio::time::sleep(duration).await;
        }
    }

    async fn fetch_product(&self) -> Result<FPSProduct, Error> {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)?;
        let query = format!("ts={}", since_the_epoch.as_millis());

        let mut url = self
            .base_url
            .clone()
            .join("/api/products/")?
            .join(&self.product)?;

        url.set_query(Some(&query));

        let response = self.client.get(url).send().await?.error_for_status()?;
        let body = response.bytes().await?;

        let product = serde_json::from_slice(&body)?;

        Ok(product)
    }
}
