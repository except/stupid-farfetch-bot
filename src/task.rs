use crate::model::FPSAddress;
use crate::model::FPSCardPaymentIntent;
use crate::model::FPSCity;
use crate::model::FPSCreateOrder;
use crate::model::FPSItem;
use crate::model::FPSOrder;
use crate::model::FPSPatchAddress;
use crate::model::FPSState;
use crate::model::Profile;
use crate::model::Variant;
use crate::Error;
use log::error;
use log::info;
use rand::prelude::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT_LANGUAGE, HOST, USER_AGENT};
use reqwest::Client;
use reqwest::Url;
use strum::AsStaticRef;
use tokio::sync::broadcast::Receiver;

#[derive(Debug)]
pub struct Task {
    client: Client,
    profile: Profile,
    base_url: Url,
    payment_client: Client,
    receiver: Receiver<(i64, Vec<Variant>)>,
}

impl Task {
    pub fn new(profile: Profile, receiver: Receiver<(i64, Vec<Variant>)>) -> Result<Task, Error> {
        let country = &profile.delivery.country;
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

        let mut headers = HeaderMap::new();
        headers.insert(
            HOST,
            HeaderValue::from_static("fps-farfetch-payment-gateway.farfetch.net"),
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0",
            ),
        );
        headers.insert("True-Client-IP", HeaderValue::from_static("127.0.0.1"));

        let payment_client = Client::builder()
            .use_rustls_tls()
            .gzip(true)
            .cookie_store(true)
            .default_headers(headers)
            .user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 14_4_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1")
            .build()?;

        Ok(Task {
            client,
            profile,
            receiver,
            base_url,
            payment_client,
        })
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        for retry in 0..=5 {
            match self.create_session().await {
                Ok(_) => break,
                Err(why) => match retry {
                    5 => return Err(why),
                    _ => continue,
                },
            }
        }

        info!("email={} message=\"created session\"", &self.profile.email);

        let mut rng = SmallRng::from_entropy();

        let (product, variants) = self.receiver.recv().await?;

        loop {
            let mut order_id = 0;
            // let mut payment_intent: String = "".into();

            for i in 0..=10 {
                if let Some(variant) = variants.choose(&mut rng).cloned() {
                    // dbg!(&variant);
                    match self.create_order(product, variant).await {
                        Ok(order) => {
                            order_id = order.id;
                            // payment_intent = order.checkout_order.payment_intent_id;

                            info!("order={} message=\"created order\"", order_id);

                            break;
                        }
                        Err(why) => match i {
                            10 => return Err(why),
                            _ => continue,
                        },
                    }
                } else {
                    error!(
                        "email={} message=\"failed to get variant\"",
                        &self.profile.email
                    );
                    return Err(Error::Unknown("missing variant".into()));
                }
            }

            for i in 0..=10 {
                match self.patch_address(order_id).await {
                    Ok(_) => {
                        info!("order={} message=\"patched address\"", order_id);
                        break;
                    }
                    Err(why) => match i {
                        10 => return Err(why),
                        _ => continue,
                    },
                }
            }

            match self.submit_payment(order_id).await {
                Ok(_) => {
                    info!("order={} message=\"submitted order\"", order_id);
                }
                Err(_) => {
                    error!("order={} message=\"failed to submit order\"", order_id);
                }
            }
        }

        // Ok(())
    }

    async fn create_session(&self) -> Result<(), Error> {
        let url = self.base_url.join("/api/users/me")?;
        self.client.get(url).send().await?.error_for_status()?;

        Ok(())
    }

    async fn create_order(&self, product: i64, variant: Variant) -> Result<FPSOrder, Error> {
        let url = {
            let mut url = self.base_url.clone();
            url.set_path("/api/checkout/v1/orders");
            url
        };

        let body = FPSCreateOrder {
            guest_user_email: &self.profile.email,
            use_payment_intent: false,
            shipping_mode: "byMerchant",
            items: vec![FPSItem {
                merchant_id: variant.merchant_id,
                variant_id: variant.id,
                product_id: product,
                quantity: 1,
            }],
        };

        let response = self
            .client
            .post(url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let body = response.bytes().await?;
        let order: FPSOrder = serde_json::from_slice(&body)?;

        Ok(order)
    }

    async fn patch_address(&self, order: i64) -> Result<(), Error> {
        let url = {
            let mut url = self.base_url.clone();
            url.set_path("/api/checkout/v1/orders/");
            url.join(&order.to_string())?
        };

        let billing_address = FPSAddress {
            first_name: &self.profile.billing.first_name,
            last_name: &self.profile.billing.last_name,
            country: self.profile.billing.country.fps_country(),
            address_line1: &self.profile.billing.address1,
            address_line2: &self.profile.billing.address2.clone().unwrap_or("".into()),
            address_line3: "",
            city: FPSCity {
                name: &self.profile.billing.city,
            },
            state: FPSState {
                name: &self.profile.billing.state.clone().unwrap_or("".into()),
            },
            zip_code: &self.profile.billing.zip,
            phone: &self.profile.phone,
        };

        let shipping_address = FPSAddress {
            first_name: &self.profile.delivery.first_name,
            last_name: &self.profile.delivery.last_name,
            country: self.profile.delivery.country.fps_country(),
            address_line1: &self.profile.delivery.address1,
            address_line2: &self.profile.delivery.address2.clone().unwrap_or("".into()),
            address_line3: "",
            city: FPSCity {
                name: &self.profile.delivery.city,
            },
            state: FPSState {
                name: &self.profile.delivery.state.clone().unwrap_or("".into()),
            },
            zip_code: &self.profile.delivery.zip,
            phone: &self.profile.phone,
        };

        let body = FPSPatchAddress {
            billing_address,
            shipping_address,
        };

        let response = self
            .client
            .patch(url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let body = response.bytes().await?;
        let _order: FPSOrder = serde_json::from_slice(&body)?;

        Ok(())
    }

    // async fn fetch_payment_form(&self, payment_intent: &str) -> Result<(String, String), Error> {
    //     let url = Url::parse_with_params(
    //         "https://org-pay-br15k8.farfetch.net",
    //         &[
    //             ("paymentIntentId", payment_intent),
    //             ("staticName", "emiliopucci"),
    //             ("folderName", "ep-21"),
    //             ("locale", "en-US"),
    //         ],
    //     )?;

    //     let response = self
    //         .payment_client
    //         .get(url)
    //         .send()
    //         .await?
    //         .error_for_status()?;

    //     let text = response.text().await?;
    //     let split = text.split("var sessionId = '").collect::<Vec<_>>();
    //     let second_split = split[1].split("';").collect::<Vec<_>>();

    //     let third_split = text.split("name=\"_csrf\" value=\"").collect::<Vec<_>>();
    //     let fourth_split = third_split[1].split("\"").collect::<Vec<_>>();

    //     let session = second_split[0].to_string();
    //     let csrf = fourth_split[0].to_string();

    //     Ok((session, csrf))
    // }

    // async fn create_payment_intent(&self, session: &str, csrf: &str) -> Result<String, Error> {
    //     let url = Url::parse_with_params(
    //         "https://org-pay-br15k8.farfetch.net/instruments",
    //         &[("sessionId", session)],
    //     )?;

    //     let holder_name = format!(
    //         "{} {}",
    //         &self.profile.billing.first_name, &self.profile.billing.last_name
    //     );

    //     let intent = FPSCardPaymentIntent {
    //         card_number: &self.profile.card.number,
    //         card_holder_name: &holder_name,
    //         card_expiry_month: &self.profile.card.expiry_month,
    //         card_expiry_year: &self.profile.card.expiry_year,
    //         card_cvv: &self.profile.card.cvv,
    //     };

    //     // "paymentMethodType": "CreditCard",
    //                                         // "paymentMethodId": "e13bb06b-392b-49a0-8acd-3f44416e3234",
    //                                         // "savePaymentMethodAsToken": True

    //     let response = self
    //         .payment_client
    //         .post(url)
    //         .json(&intent)
    //         .header("X-CSRF-TOKEN", csrf)
    //         .send()
    //         .await?
    //         .error_for_status()?;

    //     let body = response.bytes().await?;
    //     let created_intent: FPSCreatedIntent = serde_json::from_slice(&body)?;

    //     dbg!(&created_intent);

    //     Ok(created_intent.created_at)
    // }

    // async fn append_payment_intent(&self, order: i64, intent: &str) -> Result<(), Error> {
    //     let url = {
    //         let mut url = self.base_url.clone();
    //         let path = format!("/api{}", intent);
    //         url.set_path(&path);

    //         url
    //     };

    // let response = self
    //     .payment_client
    //     .post(url)
    //     .send()
    //     .await?
    //     .error_for_status()?;

    //     dbg!(&response);
    //     dbg!(response.text().await);
    //     panic!();

    //     Ok(())
    // }

    async fn submit_payment(&self, order: i64) -> Result<(), Error> {
        let url = {
            let mut url = self.base_url.clone();
            let path = format!("/api/checkout/v1/orders/{}/finalize", order);
            url.set_path(&path);

            url
        };

        let holder_name = format!(
            "{} {}",
            &self.profile.billing.first_name, &self.profile.billing.last_name
        );

        let card = FPSCardPaymentIntent {
            card_number: &self.profile.card.number,
            card_holder_name: &holder_name,
            card_expiry_month: self.profile.card.expiry_month,
            card_expiry_year: self.profile.card.expiry_year,
            card_cvv: &self.profile.card.cvv,
            payment_method_type: "CreditCard",
            payment_method_id: "e13bb06b-392b-49a0-8acd-3f44416e3234",
            save_payment_method_as_token: true,
        };

        let response = self
            .client
            .post(url)
            .json(&card)
            .send()
            .await?
            .error_for_status()?;

        dbg!(&response);

        if let Ok(text) = response.text().await {
            dbg!(text);
        }

        Ok(())
    }
}
