//! Core logic for CMU bus sign backend.
//!
//! Communicates with the Pittsburgh Regional Transit (PRT) API and
//! OpenWeatherMap, serving cleaned data to:
//! - `GET /predictions`
//! - `GET /weather`
//!
//! Stale cache mechanisms are in place to respect upstream rate limits.

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use chrono::{DateTime, Duration, FixedOffset, NaiveDateTime, TimeZone, Utc};
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

/// Time (s) between prediction cache refreshes.
const CACHE_DURATION_SECONDS: i64 = 20;

/// OpenWeather Current Weather + 5 Day / 3 Hour Forecast base URL.
const OPENWEATHER_BASE_URL: &str = "https://api.openweathermap.org/data/2.5";
/// CMU campus (Pittsburgh) coordinates.
const WEATHER_LAT: f64 = 40.443_3;
const WEATHER_LON: f64 = -79.942_8;
/// Time (s) between weather cache refreshes.
const WEATHER_CACHE_DURATION_SECONDS: i64 = 600;

/// Global application state, shared across all HTTP requests.
#[derive(Clone)]
pub struct AppState {
    prt_api_key: String,
    openweather_api_key: String,
    client: reqwest::Client,
    cache: Arc<Mutex<Cache>>,
    weather_cache: Arc<Mutex<WeatherCache>>,
}

impl AppState {
    /// Initializes a new application state with empty caches.
    pub fn new(prt_api_key: String, openweather_api_key: String) -> Self {
        Self {
            prt_api_key,
            openweather_api_key,
            client: reqwest::Client::new(),
            cache: Arc::new(Mutex::new(Cache {
                last_update: None,
                data: HashMap::new(),
            })),
            weather_cache: Arc::new(Mutex::new(WeatherCache {
                last_update: None,
                data: None,
            })),
        }
    }
}

/// Stores the most recent successful predictions response and its timestamp.
struct Cache {
    last_update: Option<DateTime<Utc>>,
    data: FrontendResponse,
}

/// Stores the most recent successful weather response and its timestamp.
struct WeatherCache {
    last_update: Option<DateTime<Utc>>,
    data: Option<WeatherResponse>,
}

/// Represents various app failures when processing a request.
#[derive(Debug)]
enum AppError {
    /// A network/connection error occurred while contacting an upstream API.
    Upstream(reqwest::Error),
    /// An upstream API returned bad JSON; data that does not match our structs.
    Json(serde_json::Error),
    /// The PRT API returned OK, but embedded an error message in the payload.
    PrtApi(String),
    /// OpenWeather returned an error status or unusable payload.
    OpenWeather(String),
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
            AppError::OpenWeather(msg) => (
                StatusCode::BAD_GATEWAY,
                format!("OpenWeather API Error: {}", msg),
            ),
        };
        (status, Json(serde_json::json!({ "error": error_message }))).into_response()
    }
}

// --- INCOMING DATA (From PRT API) ---

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

// --- INCOMING DATA (From OpenWeather) ---

#[derive(Deserialize, Debug)]
struct OwmWeatherResponse {
    weather: Vec<OwmWeatherCondition>,
    main: OwmMain,
    dt: i64,
}

#[derive(Deserialize, Debug)]
struct OwmWeatherCondition {
    main: String,
    description: String,
    icon: String,
}

#[derive(Deserialize, Debug)]
struct OwmMain {
    temp: f64,
    temp_min: f64,
    temp_max: f64,
}

#[derive(Deserialize, Debug)]
struct OwmForecastResponse {
    list: Vec<OwmForecastItem>,
    city: OwmCity,
}

#[derive(Deserialize, Debug)]
struct OwmCity {
    timezone: i32,
}

#[derive(Deserialize, Debug)]
struct OwmForecastItem {
    dt: i64,
    main: OwmMain,
    weather: Vec<OwmWeatherCondition>,
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

/// Cleaned weather payload for the header.
#[derive(Serialize, Debug, Clone)]
struct WeatherResponse {
    /// OpenWeather icon code (e.g. "10d").
    icon: String,
    /// Human-readable condition (e.g. "Heavy Intensity Rain").
    condition: String,
    /// When the current precip condition is expected to end, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    condition_until: Option<String>,
    /// Current temperature in Fahrenheit.
    temp_f: i32,
    /// Today's high in Fahrenheit.
    high_f: i32,
    /// Today's low in Fahrenheit.
    low_f: i32,
}

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
        BASE_URL, state.prt_api_key, STOPS, TIME_RES, FEED_NAME
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

/// Handler for the `GET /weather` route.
///
/// Returns current conditions for CMU using OpenWeather Current Weather and
/// 5 Day / 3 Hour Forecast. Cached for `WEATHER_CACHE_DURATION_SECONDS`.
/// Falls back to stale cache if upstream fails after a successful fetch.
async fn get_weather(State(state): State<AppState>) -> Result<Json<WeatherResponse>, AppError> {
    {
        let cache = state.weather_cache.lock().await;
        if let (Some(last_update), Some(data)) = (cache.last_update, cache.data.as_ref()) {
            let elapsed = Utc::now().signed_duration_since(last_update);
            if elapsed < Duration::seconds(WEATHER_CACHE_DURATION_SECONDS) {
                println!("Returning cached weather");
                return Ok(Json(data.clone()));
            }
        }
    }

    println!("Fetching weather from OpenWeather");
    match fetch_weather(&state).await {
        Ok(weather) => {
            let mut cache = state.weather_cache.lock().await;
            cache.data = Some(weather.clone());
            cache.last_update = Some(Utc::now());
            Ok(Json(weather))
        }
        Err(err) => {
            let mut cache = state.weather_cache.lock().await;
            if let Some(data) = cache.data.clone() {
                eprintln!("OpenWeather failed, falling back to stale weather cache: {err:?}");
                cache.last_update = Some(Utc::now());
                Ok(Json(data))
            } else {
                Err(err)
            }
        }
    }
}

async fn fetch_weather(state: &AppState) -> Result<WeatherResponse, AppError> {
    let weather_url = format!(
        "{}/weather?lat={}&lon={}&units=imperial&appid={}",
        OPENWEATHER_BASE_URL, WEATHER_LAT, WEATHER_LON, state.openweather_api_key
    );
    let forecast_url = format!(
        "{}/forecast?lat={}&lon={}&units=imperial&appid={}",
        OPENWEATHER_BASE_URL, WEATHER_LAT, WEATHER_LON, state.openweather_api_key
    );

    let current = fetch_json::<OwmWeatherResponse>(&state.client, &weather_url).await?;
    let forecast = fetch_json::<OwmForecastResponse>(&state.client, &forecast_url).await?;

    let condition = current
        .weather
        .first()
        .ok_or_else(|| AppError::OpenWeather("missing weather condition".into()))?;

    let temp_f = current.main.temp.round() as i32;
    let (high_f, low_f) = daily_high_low(&current, &forecast);
    let condition_until = precip_until(condition, &current, &forecast);

    Ok(WeatherResponse {
        icon: condition.icon.clone(),
        condition: title_case(&condition.description),
        condition_until,
        temp_f,
        high_f,
        low_f,
    })
}

async fn fetch_json<T: for<'de> Deserialize<'de>>(
    client: &reqwest::Client,
    url: &str,
) -> Result<T, AppError> {
    let resp = client.get(url).send().await.map_err(AppError::Upstream)?;
    let status = resp.status();
    let body = resp.text().await.map_err(AppError::Upstream)?;

    if !status.is_success() {
        return Err(AppError::OpenWeather(format!(
            "status {status}: {}",
            body.chars().take(200).collect::<String>()
        )));
    }

    serde_json::from_str(&body).map_err(AppError::Json)
}

/// High/low for the local calendar day from current + forecast samples.
fn daily_high_low(current: &OwmWeatherResponse, forecast: &OwmForecastResponse) -> (i32, i32) {
    let tz = forecast.city.timezone;
    let today = local_date(current.dt, tz);

    let mut high = current.main.temp_max;
    let mut low = current.main.temp_min;

    for item in &forecast.list {
        if local_date(item.dt, tz) != today {
            continue;
        }
        high = high.max(item.main.temp_max).max(item.main.temp);
        low = low.min(item.main.temp_min).min(item.main.temp);
    }

    (high.round() as i32, low.round() as i32)
}

/// If currently precipitating, find when the forecast clears and format "Until 10 PM".
fn precip_until(
    condition: &OwmWeatherCondition,
    current: &OwmWeatherResponse,
    forecast: &OwmForecastResponse,
) -> Option<String> {
    if !is_precip(&condition.main) {
        return None;
    }

    let tz = forecast.city.timezone;
    let clear_at = forecast.list.iter().find_map(|item| {
        if item.dt <= current.dt {
            return None;
        }
        let main = item.weather.first().map(|w| w.main.as_str()).unwrap_or("");
        if is_precip(main) { None } else { Some(item.dt) }
    })?;

    Some(format_until(clear_at, tz))
}

fn is_precip(main: &str) -> bool {
    matches!(main, "Rain" | "Drizzle" | "Thunderstorm" | "Snow")
}

fn local_date(ts: i64, tz_offset_secs: i32) -> chrono::NaiveDate {
    let offset = FixedOffset::east_opt(tz_offset_secs).unwrap_or(FixedOffset::east_opt(0).unwrap());
    Utc.timestamp_opt(ts, 0)
        .single()
        .unwrap_or_else(Utc::now)
        .with_timezone(&offset)
        .date_naive()
}

fn format_until(ts: i64, tz_offset_secs: i32) -> String {
    let offset = FixedOffset::east_opt(tz_offset_secs).unwrap_or(FixedOffset::east_opt(0).unwrap());
    let dt = Utc
        .timestamp_opt(ts, 0)
        .single()
        .unwrap_or_else(Utc::now)
        .with_timezone(&offset);
    let hour = dt.format("%I").to_string();
    let hour = hour.trim_start_matches('0');
    format!("Until {} {}", hour, dt.format("%p"))
}

fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Constructs the Axum router and attaches the application state.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/predictions", get(get_predictions))
        .route("/weather", get(get_weather))
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
