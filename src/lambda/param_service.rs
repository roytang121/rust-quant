use crate::lambda::lambda_instance::{
    LambdaStrategyParamsRequest, LambdaStrategyParamsRequestSender,
};
use crate::lambda::LambdaState;

use rocket::data::ToByteUnit;

use rocket::http::{Header, Method};
use rocket::response::content::Json;
use rocket::route::{Handler, Outcome};

use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Data, Request, Response};
use serde_json::Value;

#[derive(Clone)]
pub struct LambdaStrategyParamService {
    strategy_param_request_sender: LambdaStrategyParamsRequestSender,
    pub route: MyRoute,
}

#[derive(Clone)]
pub enum MyRoute {
    GetParam,
    UpdateParam,
    GetState,
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyValueEntry {
    key: String,
    value: Value,
    #[serde()]
    #[serde(rename = "type")]
    type_: KeyValueType,
    group: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum KeyValueType {
    String,
    Int,
    Float,
    Bool,
}

pub trait Stated {
    fn get_state(&self) -> &LambdaState;
    fn set_state(&mut self, new_state: LambdaState);
}

pub struct CORS;

#[async_trait::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

impl LambdaStrategyParamService {
    pub fn new(strategy_param_request_sender: LambdaStrategyParamsRequestSender) -> Self {
        LambdaStrategyParamService {
            strategy_param_request_sender,
            route: MyRoute::None,
        }
    }
    pub fn route(&self, route: MyRoute) -> Self {
        let mut copy = self.clone();
        copy.route = route;
        copy
    }
    pub async fn subscribe(&self) -> Result<(), rocket::Error> {
        let config = rocket::Config {
            port: 6008,
            ..rocket::Config::debug_default()
        };
        let routes = vec![
            // params
            rocket::route::Route::new(Method::Get, "/params", self.route(MyRoute::GetParam)),
            rocket::route::Route::new(Method::Options, "/params", self.route(MyRoute::None)),
            rocket::route::Route::new(Method::Post, "/params", self.route(MyRoute::UpdateParam)),
            // states
            rocket::route::Route::new(Method::Get, "/states", self.route(MyRoute::GetState)),
        ];
        rocket::build()
            .configure(config)
            .mount("/", routes)
            .attach(CORS)
            .launch()
            .await
    }
}

#[rocket::async_trait]
impl Handler for LambdaStrategyParamService {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
        match self.route {
            MyRoute::GetParam => self.get_param_entries(request, data).await,
            MyRoute::UpdateParam => self.update_param_entries(request, data).await,
            MyRoute::GetState => self.get_state_entries(request, data).await,
            _ => self.unsupported_route(request, data).await,
        }
    }
}

impl LambdaStrategyParamService {
    async fn from_data<'r>(data: Data<'r>) -> Result<Value, Box<dyn std::error::Error>> {
        let mut bytes = data.open(512.kibibytes()).into_string().await?;
        let body: Value = serde_json::from_str(bytes.value.as_mut_str())?;
        Ok(body)
    }

    fn value_to_entries(value: &Value, group: &str) -> Vec<KeyValueEntry> {
        let mut entries: Vec<KeyValueEntry> = Vec::new();
        if let Some(obj) = value.as_object() {
            for (key, value) in obj {
                let mut entry = KeyValueEntry {
                    key: key.clone(),
                    value: value.clone(),
                    type_: KeyValueType::String,
                    group: group.to_string(),
                };
                if value.is_boolean() {
                    entry.type_ = KeyValueType::Bool;
                } else if value.is_f64() {
                    entry.type_ = KeyValueType::Float;
                } else if value.is_i64() {
                    entry.type_ = KeyValueType::Int;
                } else if value.is_string() {
                    entry.type_ = KeyValueType::String;
                }
                entries.push(entry);
            }
        }
        entries
    }

    async fn get_param_entries<'r>(
        &self,
        request: &'r Request<'_>,
        _data: Data<'r>,
    ) -> Outcome<'r> {
        let snapshot = LambdaStrategyParamsRequest::request_strategy_params_snapshot(
            &self.strategy_param_request_sender,
        )
        .await;
        match snapshot {
            Ok(value) => {
                let entries = Self::value_to_entries(&value, "params");
                let response =
                    serde_json::to_string(&entries).unwrap_or_else(|err| format!("{}", err));
                return Outcome::from(request, Json(response));
            }
            Err(err) => Outcome::from(request, format!("Error: {}", err)),
        }
        // Outcome::from(request, "Error")
    }

    async fn update_param_entries<'r>(
        &self,
        request: &'r Request<'_>,
        data: Data<'r>,
    ) -> Outcome<'r> {
        let body = Self::from_data(data).await.unwrap();
        let entries = serde_json::from_value::<Vec<KeyValueEntry>>(body).unwrap();
        let snapshot = LambdaStrategyParamsRequest::request_strategy_params_snapshot(
            &self.strategy_param_request_sender,
        )
        .await;
        match snapshot {
            Ok(mut snapshot) => {
                let mut snapshot = snapshot.as_object_mut().unwrap().clone();
                for entry in entries.into_iter() {
                    if snapshot.contains_key(entry.key.as_str()) {
                        if let Some(_current_value) = snapshot.get(entry.key.as_str()) {
                            // TODO: reject if value type does not match
                        }
                        snapshot.insert(entry.key, entry.value);
                    }
                }
                info!("{:?}", snapshot);
                let snapshot = Value::from(snapshot);
                LambdaStrategyParamsRequest::request_update_strategy_params(
                    &self.strategy_param_request_sender,
                    snapshot,
                )
                .await;
                return Outcome::from(request, "Ok");
            }
            Err(_) => Outcome::from(request, "Error"),
        };
        Outcome::from(request, "Error")
    }

    async fn get_state_entries<'r>(
        &self,
        request: &'r Request<'_>,
        _data: Data<'r>,
    ) -> Outcome<'r> {
        let snapshot = LambdaStrategyParamsRequest::request_lambda_state_snapshot(
            &self.strategy_param_request_sender,
        )
        .await;
        match snapshot {
            Ok(value) => {
                return match value {
                    None => Outcome::from(request, Json("{}")),
                    Some(value) => {
                        let entries = Self::value_to_entries(&value, "states");
                        let result = serde_json::to_string(&entries).unwrap();
                        Outcome::from(request, Json(result))
                    }
                };
                // let json = value.to_string();
                // let entries = Self::value_to_entries(&value);
                // let response =
                //     serde_json::to_string(&value).unwrap_or_else(|err| format!("{}", err));
                // return Outcome::from(request, Json(json))
            }
            Err(_) => Outcome::from(request, "Error"),
        }
    }

    async fn unsupported_route<'r>(
        &self,
        request: &'r Request<'_>,
        _data: Data<'r>,
    ) -> Outcome<'r> {
        Outcome::from(request, "Unsupported Route")
    }
}
