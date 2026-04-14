//! Core logic for CMU bus sign backend.
//!
//! Communicates with the Pittsburgh Regional Transit (PRT) API,
//! serving cleaned data to http://{API_HOST}:{API_PORT}/predictions.
//!
//! Stale cache (20 sec.) mechanism in place to respect API rate limit (10,000 calls/day).

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

/// Base URL for PRT Truetime API.
const BASE_URL: &str = "http://truetime.portauthority.org/bustime/api/v3";
/// Comma-separated list of stop IDs to query.
const STOPS: &str = "4407,7117";
/// Resolution of predicted time data ('s' for seconds).
const TIME_RES: &str = "s";
/// Specific feed name required by PRT API.
const FEED_NAME: &str = "Port Authority Bus";

/// Time (s) between cache refreshes.
const CACHE_DURATION_SECONDS: i64 = 20;

/// Global application state, shared across all HTTP requests.
#[derive(Clone)]
pub struct AppState {
    api_key: String,
    client: reqwest::Client,
    cache: Arc<Mutex<Cache>>,
}

impl AppState {
    /// Initializes a new application state with an empty cache.
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            cache: Arc::new(Mutex::new(Cache {
                last_update: None,
                data: HashMap::new(),
            })),
        }
    }
}

/// Stores the most recent successful API response and its timestamp.
struct Cache {
    last_update: Option<DateTime<Utc>>,
    data: FrontendResponse,
}

/// Represents various app failures when processing a request.
enum AppError {
    /// A network/connection error occurred while contacting the upstream PRT API.
    Upstream(reqwest::Error),
    /// The PRT API returned bad JSON; data that does not match our structs.
    Json(serde_json::Error),
    /// The PRT API returned OK, but embedded an error message in the payload.
    PrtApi(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Upstream(e) => (StatusCode::BAD_GATEWAY, format!("API Connect Error: {}", e)),
            AppError::Json(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("API Parse Error: {}", e),
            ),
            AppError::PrtApi(msg) => (StatusCode::BAD_GATEWAY, format!("PRT API Error: {}", msg)),
        };
        (status, Json(serde_json::json!({ "error": error_message }))).into_response()
    }
}

// --- INCOMING DATA (From API) ---

/// The top-level wrapper for the PRT API JSON response.
#[derive(Deserialize, Debug)]
struct PrtResponse {
    #[serde(rename = "bustime-response")]
    response: PrtBody,
}

/// The body of the PRT API response, which contains either predictions or an error.
#[derive(Deserialize, Debug)]
struct PrtBody {
    #[serde(rename = "prd", default)]
    predictions: Option<Vec<PrtPrediction>>,
    #[serde(rename = "error", default)]
    api_error: Option<Vec<PrtError>>,
}

/// An error message returned by the PRT API.
#[derive(Deserialize, Debug)]
struct PrtError {
    msg: String,
}

/// A single bus arrival prediction from the PRT API.
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

/// A group of all incoming buses for a specific route and destination.
#[derive(Serialize, Debug, Clone)]
struct RouteGroup {
    /// Route number/name (e.g. "61A").
    route: String,
    /// Final destination (e.g. "North Braddock").
    destination: String,
    /// A list of upcoming arrivals for this specific route and destination.
    arrivals: Vec<BusArrival>,
}

/// Represents a single bus arriving at a stop.
#[derive(Serialize, Debug, Clone)]
struct BusArrival {
    /// The unique vehicle ID.
    bus_id: String,
    /// Seconds remaining until the bus arrives.
    seconds: i64,
    /// How full the bus is.
    capacity: String,
}

/// The final payload sent to the frontend.
/// Keys = Stop IDs, values = list of grouped routes arriving at that stop.
type FrontendResponse = HashMap<String, Vec<RouteGroup>>;

/// Handler for the `GET /predictions` route.
///
/// If valid cached data exists (less than `CACHE_DURATION_SECONDS` old), it returns the cache
/// while linearly subtracting the elapsed time from the arrival predictions.
/// If the cache is stale, it queries the PRT API. If the API fails, it falls back to the
/// stale cache to prevent the bus sign from blanking out during minor network drops.
///
/// ## Errors
/// Returns an `AppError` if the upstream API fails to connect, returns bad JSON,
/// or returns an API-level error *and* no cache is available to fall back on.
async fn get_predictions(
    State(state): State<AppState>,
) -> Result<Json<FrontendResponse>, AppError> {
    // 1. check cache first
    {
        let cache = state.cache.lock().await;
        if let Some(last_update) = cache.last_update {
            let now = Utc::now();
            let elapsed = now.signed_duration_since(last_update);
            if elapsed < Duration::seconds(CACHE_DURATION_SECONDS) {
                println!("Returning cached data");

                let mut response_data = cache.data.clone();
                let elapsed_seconds = elapsed.num_seconds();

                adjust_cached_times(&mut response_data, elapsed_seconds);

                return Ok(Json(response_data));
            }
        }
    }

    // 2. fetch new data from API
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
        .map_err(AppError::Upstream)?;

    // PRT API sometimes returns escaped slashes that break JSON parsing
    let raw_text = resp.text().await.map_err(AppError::Upstream)?;
    let clean_text = raw_text.replace(r"\", "/");
    let prt_data: PrtResponse = serde_json::from_str(&clean_text).map_err(AppError::Json)?;

    // 3. handle PRT errors
    if let Some(errors) = prt_data.response.api_error {
        let combined_msg = errors
            .into_iter()
            .map(|e| e.msg)
            .collect::<Vec<String>>()
            .join(", ");

        eprintln!("PRT API Error Message: {}", combined_msg);

        let mut cache = state.cache.lock().await;

        // STALE CACHE FALLBACK LOGIC -
        // if API returns an error but we have old data, send the old data anyway
        // so the physical sign doesn't go blank
        if let Some(last_update) = cache.last_update {
            eprintln!("API failed, falling back to stale cache data");

            let now = Utc::now();
            let elapsed_seconds = now.signed_duration_since(last_update).num_seconds();

            // update actual cache data here so that next cache pull doesn't use old data
            adjust_cached_times(&mut cache.data, elapsed_seconds);

            // update timestamp and return Ok (with stale data)
            cache.last_update = Some(now);
            return Ok(Json(cache.data.clone()));
        }

        // if no cache exists yet, update timestamp to avoid API spam and return Err
        cache.last_update = Some(Utc::now());
        return Err(AppError::PrtApi(combined_msg));
    }

    // 4. parse API data into frontend format
    let mut output: FrontendResponse = HashMap::new();

    if let Some(predictions) = prt_data.response.predictions {
        for p in predictions.into_iter() {
            let format = "%Y%m%d %H:%M:%S";

            // calculate seconds remaining via predicted time - generation time
            let seconds_left = match (
                NaiveDateTime::parse_from_str(&p.tmstmp, format),
                NaiveDateTime::parse_from_str(&p.prdtm, format),
            ) {
                (Ok(s), Ok(a)) => a.signed_duration_since(s).num_seconds(),
                _ => continue, // skip prediction if time format is bad
            };

            let stop_list = output.entry(p.stpid).or_default();

            let arrival = BusArrival {
                bus_id: p.vid,
                seconds: seconds_left,
                capacity: p.psgld,
            };

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

    // 5. update cache
    {
        let mut cache = state.cache.lock().await;
        cache.data = output.clone();
        cache.last_update = Some(Utc::now());
    }

    Ok(Json(output))
}

/// Constructs the Axum router and attaches the application state.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/predictions", get(get_predictions))
        .with_state(state)
}

/// Linearly decreases predicted arrival times based on how much real time has elapsed.
/// Stops subtracting if the bus is extremely close (under 30s) to prevent negative times.
fn adjust_cached_times(data: &mut FrontendResponse, elapsed_seconds: i64) {
    for route_groups in data.values_mut() {
        for group in route_groups {
            for arrival in &mut group.arrivals {
                if arrival.seconds > 30 {
                    arrival.seconds -= elapsed_seconds;
                }
            }
        }
    }
}
