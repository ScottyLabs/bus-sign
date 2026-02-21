// BACKEND for CMU bus sign
// serves data to http://{API_HOST}:{API_PORT}/predictions
// 20-second cache in place to prevent API abuse
// (only requests from API every 20 seconds)

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

// parts of API request URL
const BASE_URL: &str = "http://truetime.portauthority.org/bustime/api/v3";
const STOPS: &str = "4407,7117"; // stops to retrieve data from
const TIME_RES: &str = "s"; // resolution of time data (seconds)
const FEED_NAME: &str = "Port Authority Bus";

// time between cache refreshes
const CACHE_DURATION_SECONDS: i64 = 20;

#[derive(Clone)]
struct AppState {
    api_key: String,
    client: reqwest::Client,
    cache: Arc<Mutex<Cache>>,
}

struct Cache {
    last_update: Option<DateTime<Utc>>,
    data: FrontendResponse,
}

enum AppError {
    UpstreamError(reqwest::Error),
    JsonError(serde_json::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::UpstreamError(e) => {
                (StatusCode::BAD_GATEWAY, format!("API Connect Error: {}", e))
            }
            AppError::JsonError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("API Parse Error: {}", e),
            ),
        };
        (status, Json(serde_json::json!({ "error": error_message }))).into_response()
    }
}

// --- INCOMING DATA (From API) ---
#[derive(Deserialize, Debug)]
struct PrtResponse {
    #[serde(rename = "bustime-response")]
    response: PrtBody,
}

#[derive(Deserialize, Debug)]
struct PrtBody {
    #[serde(rename = "prd", default)]
    predictions: Option<Vec<PrtPrediction>>,
    #[serde(rename = "error", default)]
    api_error: Option<Vec<PrtError>>,
}

#[derive(Deserialize, Debug)]
struct PrtError {
    msg: String,
}

#[derive(Deserialize, Debug)]
struct PrtPrediction {
    rt: String,
    des: String,
    stpid: String,
    vid: String,
    tmstmp: String,
    prdtm: String,
    #[serde(default)]
    psgld: String,
}

// --- OUTGOING DATA (To Frontend) ---
#[derive(Serialize, Debug, Clone)]
struct RouteGroup {
    route: String,
    destination: String,
    arrivals: Vec<BusArrival>,
}

#[derive(Serialize, Debug, Clone)]
struct BusArrival {
    bus_id: String,
    seconds: i64,
    capacity: String,
}

type FrontendResponse = HashMap<String, Vec<RouteGroup>>;

#[tokio::main]
async fn main() {
    // load API key from .env in parent directory
    dotenv().ok();

    let api_key = env::var("PRT_API_KEY").expect("PRT_API_KEY must be set in .env");

    let state = AppState {
        api_key,
        client: reqwest::Client::new(),
        cache: Arc::new(Mutex::new(Cache {
            last_update: None,
            data: HashMap::new(),
        })),
    };

    // cors for security - allow(Any) is fine for this but not best practice (fix before prod)
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any);

    let app = Router::new()
        .route("/predictions", get(get_predictions))
        .layer(cors)
        .with_state(state);

    let host: String = env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let ip: IpAddr = host.parse().expect("API_HOST must be a valid IP address");

    let port: u16 = env::var("API_PORT")
        .unwrap_or_else(|_| "8080".to_string()) // default port 8080 if error
        .parse()
        .expect("API_PORT must be a valid port number");

    let addr = SocketAddr::from((ip, port));
    println!("Server started on http://{}/predictions", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_predictions(
    State(state): State<AppState>,
) -> Result<Json<FrontendResponse>, AppError> {
    {
        let cache = state.cache.lock().await;
        if let Some(last_update) = cache.last_update {
            let now = Utc::now();
            let elapsed = now.signed_duration_since(last_update);
            if elapsed < Duration::seconds(CACHE_DURATION_SECONDS) {
                println!("Returning cached data");

                let mut response_data = cache.data.clone();

                let elapsed_seconds = elapsed.num_seconds();

                // if pulling from cache, linearly decreases predicted times according to real time elapsed
                for route_groups in response_data.values_mut() {
                    for group in route_groups {
                        for arrival in &mut group.arrivals {
                            if arrival.seconds > 30 {
                                arrival.seconds -= elapsed_seconds;
                            }
                        }
                    }
                }

                return Ok(Json(response_data));
            }
        }
    }

    println!("Fetching from API");
    let url = format!(
        "{}/getpredictions?key={}&stpid={}&tmres={}&rtpidatafeed={}&format=json",
        BASE_URL, state.api_key, STOPS, TIME_RES, FEED_NAME
    );

    let resp = state
        .client
        .get(&url)
        .send()
        .await
        .map_err(AppError::UpstreamError)?;

    let raw_text = resp.text().await.map_err(AppError::UpstreamError)?;
    let clean_text = raw_text.replace(r"\", "/");
    let prt_data: PrtResponse = serde_json::from_str(&clean_text).map_err(AppError::JsonError)?;

    if let Some(errors) = prt_data.response.api_error {
        for err in errors {
            println!("PRT API Error Message: {}", err.msg);
        }
        return Ok(Json(HashMap::new()));
    }

    let mut output: FrontendResponse = HashMap::new();

    if let Some(predictions) = prt_data.response.predictions {
        // handle API data
        for p in predictions.into_iter() {
            let format = "%Y%m%d %H:%M:%S";
            let seconds_left = match (
                NaiveDateTime::parse_from_str(&p.tmstmp, format),
                NaiveDateTime::parse_from_str(&p.prdtm, format),
            ) {
                (Ok(s), Ok(a)) => a.signed_duration_since(s).num_seconds(),
                _ => continue, // skip this iteration if bad time data
                               // TODO: add some kind of internal warning here
            };

            let stop_list = output.entry(p.stpid).or_default();

            // data for each bus
            let arrival = BusArrival {
                bus_id: p.vid,
                seconds: seconds_left,
                capacity: p.psgld,
            };

            // if stop data already exists, update it; otherwise, make new
            if let Some(group) = stop_list
                .iter_mut()
                .find(|g| g.route == p.rt && g.destination == p.des)
            {
                group.arrivals.push(arrival);
                group.arrivals.sort_by_key(|b| b.seconds);
            } else {
                stop_list.push(RouteGroup {
                    route: p.rt,
                    destination: p.des,
                    arrivals: vec![arrival],
                });
            }
        }
    }

    {
        let mut cache = state.cache.lock().await;
        cache.data = output.clone();
        cache.last_update = Some(Utc::now());
    }

    Ok(Json(output))
}
