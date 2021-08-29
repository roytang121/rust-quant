use crate::core::config::ConfigStore;
use crate::ftx::types::FtxPlaceOrder;
use crate::model::OrderRequest;
use hmac::{Hmac, Mac, NewMac};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use reqwest::{RequestBuilder, Response};
use sha2::Sha256;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use url::Url;

pub struct FtxRestClient {
    base_url: String,
    client: reqwest::Client,
    secret: String,
}

type ApiResult = Result<serde_json::Value, Box<dyn std::error::Error>>;

impl FtxRestClient {
    pub fn new() -> FtxRestClient {
        let config = ConfigStore::load();
        let builder = reqwest::ClientBuilder::new();
        let mut headers = HeaderMap::new();
        headers.insert("FTX-KEY", config.ftx_api_key.parse().unwrap());
        headers.insert("FTX-SUBACCOUNT", config.ftx_sub_account.parse().unwrap());
        let client = builder.default_headers(headers).build().unwrap();
        let base_url = "https://ftx.com/".to_string();
        FtxRestClient {
            client,
            base_url,
            secret: config.ftx_api_secret,
        }
    }

    fn hashmap_to_entries(map: &HashMap<String, String>) -> Vec<(&String, &String)> {
        let mut entires = vec![];
        for (k, v) in map {
            entires.push((k, v))
        }
        entires
    }

    pub fn generate_signature(
        secret: &str,
        ts: i64,
        method: &str,
        request_path: &str,
        params: Option<&str>,
        json_body: Option<&str>,
    ) -> String {
        type HmacSha256 = Hmac<Sha256>;
        // TS implementation
        // const sign = CryptoJS.HmacSHA256(`${ts}websocket_login`, secret).toString();
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        let payload = match method {
            "GET" | "DELETE" => match params {
                None => format!(
                    "{ts}{method}{request_path}",
                    ts = ts,
                    method = method,
                    request_path = request_path
                ),
                Some(params) => format!(
                    "{ts}{method}{request_path}?{params}",
                    ts = ts,
                    method = method,
                    request_path = request_path,
                    params = params
                ),
            },
            "POST" => match json_body {
                None => format!(
                    "{ts}{method}{request_path}",
                    ts = ts,
                    method = method,
                    request_path = request_path
                ),
                Some(json_body) => format!(
                    "{ts}{method}{request_path}{json_body}",
                    ts = ts,
                    method = method,
                    request_path = request_path,
                    json_body = json_body
                ),
            },
            _ => {
                panic!("Unknown HTTP method")
            }
        };
        log::debug!("{}", payload);
        mac.update(payload.as_bytes());

        let result = mac.finalize().into_bytes();

        return hex::encode(result);
    }

    pub fn get(&self, path: &str, params: Option<HashMap<String, String>>) -> RequestBuilder {
        let ts = chrono::Utc::now().timestamp_millis();
        let request_path = format!("{base_url}api{path}", base_url = self.base_url, path = path);
        let url = match params {
            None => reqwest::Url::parse(request_path.as_str()).unwrap(),
            Some(ref params) => {
                let entries = Self::hashmap_to_entries(params);
                reqwest::Url::parse_with_params(request_path.as_str(), entries).unwrap()
            }
        };
        let query = match params {
            None => None,
            Some(_) => url.query(),
        };
        let sign = Self::generate_signature(&self.secret, ts, "GET", url.path(), query, None);
        let request = self
            .client
            .get(url)
            .header("FTX-TS", ts)
            .header("FTX-SIGN", sign);
        request
    }

    pub fn post(&self, path: &str, json: serde_json::Value) -> RequestBuilder {
        let ts = chrono::Utc::now().timestamp_millis();
        let request_path = format!("{base_url}api{path}", base_url = self.base_url, path = path);
        let url = reqwest::Url::parse(request_path.as_str()).unwrap();
        let json_body = serde_json::to_string(&json).unwrap();
        log::info!("{}", json_body);
        let sign = Self::generate_signature(
            &self.secret,
            ts,
            "POST",
            url.path(),
            None,
            Option::from(json_body.as_str()),
        );
        let request = self
            .client
            .post(url)
            .body(json_body)
            .header(CONTENT_TYPE, "application/json")
            .header("FTX-TS", ts)
            .header("FTX-SIGN", sign);
        request
    }

    pub fn delete(&self, path: &str, params: Option<HashMap<String, String>>) -> RequestBuilder {
        let ts = chrono::Utc::now().timestamp_millis();
        let request_path = format!("{base_url}api{path}", base_url = self.base_url, path = path);
        let url = match params {
            None => reqwest::Url::parse(request_path.as_str()).unwrap(),
            Some(ref params) => {
                let entries = Self::hashmap_to_entries(params);
                reqwest::Url::parse_with_params(request_path.as_str(), entries).unwrap()
            }
        };
        let query = match params {
            None => None,
            Some(_) => url.query(),
        };
        let sign = Self::generate_signature(&self.secret, ts, "DELETE", url.path(), query, None);
        let request = self
            .client
            .delete(url)
            .header("FTX-TS", ts)
            .header("FTX-SIGN", sign);
        request
    }
}

// API methods
impl FtxRestClient {
    pub async fn get_account(&self) -> Result<String, Box<dyn std::error::Error>> {
        let request = self.get("/account", None);
        let response = request.send().await?;
        let json = response.text().await?;
        Ok(json)
    }

    pub async fn get_fills(&self, market: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut params = HashMap::new();
        params.insert(String::from("market"), market.to_string());
        let request = self.get("/fills", Option::from(params));
        let response = request.send().await?;
        let json = response.text().await?;
        Ok(json)
    }

    pub async fn place_order(&self, order: OrderRequest) -> ApiResult {
        let ftx_request = FtxPlaceOrder::from_order_request(order);
        let json = serde_json::to_value(ftx_request)?;
        let request = self.post("/orders", json);
        let response = request.send().await?;
        let json = response.json::<serde_json::Value>().await?;
        Ok(json)
    }

    pub async fn cancel_order(&self, order_id: i64) -> ApiResult {
        let request = self.delete(format!("/orders/{}", order_id).as_str(), None);
        let response = request.send().await?;
        let json = response.json::<serde_json::Value>().await?;
        Ok(json)
    }

    pub async fn cancel_order_cid(&self, cid: &str) -> ApiResult {
        let request = self.delete(format!("/orders/by_client_id/{}", cid).as_str(), None);
        let response = request.send().await?;
        let json = response.json::<serde_json::Value>().await?;
        Ok(json)
    }
}
