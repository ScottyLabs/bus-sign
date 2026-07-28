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
use chrono::{DateTime, Datelike, Duration, Local, NaiveDateTime, Utc, Weekday};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::{collections::HashMap, env, sync::Arc};
use tokio::{signal, sync::Mutex};
use tower_http::cors::{Any, CorsLayer};

// parts of API request URL
const BASE_URL: &str = "http://truetime.portauthority.org/bustime/api/v3";
const STOPS: &str = "4407,7117"; // stops to retrieve data from
const TIME_RES: &str = "s"; // resolution of time data (seconds)
const FEED_NAME: &str = "Port Authority Bus";

// time between cache refreshes
const PREDICTIONS_CACHE_DURATION_SECONDS: i64 = 20;
const WEATHER_CACHE_DURATION_SECONDS: i64 = 15 * 60;
const WEATHER_URL: &str = "https://api.open-meteo.com/v1/forecast";
const WEATHER_LATITUDE: &str = "40.4444";
const WEATHER_LONGITUDE: &str = "-79.9436";
const WEATHER_TIMEZONE: &str = "America/New_York";
const TWENTY_EIGHT_X_SCHEDULE_URL: &str = "https://www.rideprt.org/pdfs/28X.pdf";
const TWENTY_EIGHT_X_SCHEDULE_CACHE_DURATION_SECONDS: i64 = 7 * 24 * 60 * 60;

#[derive(Clone)]
struct AppState {
    api_key: String,
    client: reqwest::Client,
    predictions_cache: Arc<Mutex<PredictionsCache>>,
    weather_cache: Arc<Mutex<WeatherCache>>,
    twenty_eight_x_schedule_cache: Arc<Mutex<TwentyEightXScheduleCache>>,
}

struct PredictionsCache {
    last_update: Option<DateTime<Utc>>,
    data: FrontendResponse,
}

struct WeatherCache {
    last_update: Option<DateTime<Utc>>,
    data: Option<WeatherResponseData>,
}

struct TwentyEightXScheduleCache {
    last_update: Option<DateTime<Utc>>,
    data: Option<TwentyEightXScheduleResponse>,
}

enum AppError {
    UpstreamError(reqwest::Error),
    JsonError(serde_json::Error),
    ScheduleError(String),
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
            AppError::ScheduleError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Schedule Parse Error: {}", e),
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

#[derive(Deserialize, Debug)]
struct OpenMeteoResponse {
    current: OpenMeteoCurrent,
    hourly: OpenMeteoHourly,
}

#[derive(Deserialize, Debug)]
struct OpenMeteoCurrent {
    time: String,
    temperature_2m: f64,
    weather_code: i32,
    is_day: u8,
}

#[derive(Deserialize, Debug)]
struct OpenMeteoHourly {
    time: Vec<String>,
    weather_code: Vec<i32>,
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

#[derive(Serialize, Debug, Clone)]
struct WeatherResponseData {
    temperature_f: i64,
    weather_code: i32,
    is_day: bool,
    summary: String,
    detail: String,
}

#[derive(Serialize, Debug, Clone)]
struct ScheduledArrival {
    route: String,
    destination: String,
    seconds: i64,
    scheduled: bool,
}

type TwentyEightXScheduleResponse = HashMap<String, Vec<ScheduledArrival>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ServiceType {
    Weekday,
    Saturday,
    SundayHoliday,
}

#[tokio::main]
async fn main() {
    // load API key from .env in parent directory
    dotenvy::dotenv().ok();

    let api_key = env::var("PRT_API_KEY").expect("PRT_API_KEY must be set in .env");

    let state = AppState {
        api_key,
        client: reqwest::Client::new(),
        predictions_cache: Arc::new(Mutex::new(PredictionsCache {
            last_update: None,
            data: HashMap::new(),
        })),
        weather_cache: Arc::new(Mutex::new(WeatherCache {
            last_update: None,
            data: None,
        })),
        twenty_eight_x_schedule_cache: Arc::new(Mutex::new(TwentyEightXScheduleCache {
            last_update: None,
            data: None,
        })),
    };

    // cors for security - allow(Any) is fine for this but not best practice (fix before prod)
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any);

    let app = Router::new()
        .route("/predictions", get(get_predictions))
        .route("/weather", get(get_weather))
        .route("/schedule/28x", get(get_twenty_eight_x_schedule))
        .layer(cors)
        .with_state(state);

    let host: String = env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let ip: IpAddr = host.parse().expect("API_HOST must be a valid IP address");

    let port: u16 = env::var("API_PORT")
        .unwrap_or_else(|_| "8080".to_string()) // default port 8080 if error
        .parse()
        .expect("API_PORT must be a valid port number");

    let addr = SocketAddr::from((ip, port));
    println!("Server started on http://{}/predictions, /weather, and /schedule/28x", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

// Adding a handler for shutdown signals
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn get_predictions(
    State(state): State<AppState>,
) -> Result<Json<FrontendResponse>, AppError> {
    {
        let cache = state.predictions_cache.lock().await;
        if let Some(last_update) = cache.last_update {
            let now = Utc::now();
            let elapsed = now.signed_duration_since(last_update);
            if elapsed < Duration::seconds(PREDICTIONS_CACHE_DURATION_SECONDS) {
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

        {
            let mut cache = state.predictions_cache.lock().await;
            cache.data = HashMap::new();
            cache.last_update = Some(Utc::now());
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
        let mut cache = state.predictions_cache.lock().await;
        cache.data = output.clone();
        cache.last_update = Some(Utc::now());
    }

    Ok(Json(output))
}

async fn get_weather(
    State(state): State<AppState>,
) -> Result<Json<WeatherResponseData>, AppError> {
    {
        let cache = state.weather_cache.lock().await;
        if let Some(last_update) = cache.last_update {
            let now = Utc::now();
            let elapsed = now.signed_duration_since(last_update);
            if elapsed < Duration::seconds(WEATHER_CACHE_DURATION_SECONDS) {
                if let Some(weather) = &cache.data {
                    println!("Returning cached weather");
                    return Ok(Json(weather.clone()));
                }
            }
        }
    }

    println!("Fetching weather");
    let url = format!(
        "{WEATHER_URL}?latitude={WEATHER_LATITUDE}&longitude={WEATHER_LONGITUDE}&current=temperature_2m,weather_code,is_day&hourly=weather_code&temperature_unit=fahrenheit&timezone={WEATHER_TIMEZONE}&forecast_days=1"
    );

    let weather_api_data = state
        .client
        .get(&url)
        .send()
        .await
        .map_err(AppError::UpstreamError)?
        .json::<OpenMeteoResponse>()
        .await
        .map_err(AppError::UpstreamError)?;

    let weather = WeatherResponseData {
        temperature_f: weather_api_data.current.temperature_2m.round() as i64,
        weather_code: weather_api_data.current.weather_code,
        is_day: weather_api_data.current.is_day == 1,
        summary: weather_summary(weather_api_data.current.weather_code).to_string(),
        detail: weather_detail(
            &weather_api_data.current.time,
            weather_api_data.current.weather_code,
            &weather_api_data.hourly,
        ),
    };

    {
        let mut cache = state.weather_cache.lock().await;
        cache.data = Some(weather.clone());
        cache.last_update = Some(Utc::now());
    }

    Ok(Json(weather))
}

fn weather_summary(code: i32) -> &'static str {
    match code {
        0 => "Clear sky",
        1 => "Mainly clear",
        2 => "Partly cloudy",
        3 => "Overcast",
        45 | 48 => "Fog",
        51 => "Light drizzle",
        53 => "Drizzle",
        55 => "Heavy drizzle",
        56 | 57 => "Freezing drizzle",
        61 => "Light rain",
        63 => "Rain",
        65 => "Heavy rain",
        66 | 67 => "Freezing rain",
        71 => "Light snow",
        73 => "Snow",
        75 => "Heavy snow",
        77 => "Snow grains",
        80 => "Light showers",
        81 => "Showers",
        82 => "Heavy showers",
        85 | 86 => "Snow showers",
        95 => "Thunderstorm",
        96 | 99 => "Severe storm",
        _ => "Current conditions",
    }
}

fn weather_detail(current_time: &str, current_code: i32, hourly: &OpenMeteoHourly) -> String {
    let current_summary = weather_summary(current_code);
    let current_index = hourly.time.iter().position(|time| time == current_time).unwrap_or(0);
    let last_index = hourly.time.len().min(hourly.weather_code.len());

    for index in (current_index + 1)..last_index {
        if weather_summary(hourly.weather_code[index]) != current_summary {
            if let Ok(next_time) = NaiveDateTime::parse_from_str(&hourly.time[index], "%Y-%m-%dT%H:%M")
            {
                let formatted = next_time.format("%I %p").to_string();
                return format!("Until {}", formatted.trim_start_matches('0'));
            }
        }
    }

    "Rest of day".to_string()
}

async fn get_twenty_eight_x_schedule(
    State(state): State<AppState>,
) -> Result<Json<TwentyEightXScheduleResponse>, AppError> {
    {
        let cache = state.twenty_eight_x_schedule_cache.lock().await;
        if let Some(last_update) = cache.last_update {
            let now = Utc::now();
            let elapsed = now.signed_duration_since(last_update);
            if elapsed < Duration::seconds(TWENTY_EIGHT_X_SCHEDULE_CACHE_DURATION_SECONDS) {
                if let Some(schedule) = &cache.data {
                    println!("Returning cached 28X schedule");
                    return Ok(Json(schedule.clone()));
                }
            }
        }
    }

    println!("Fetching 28X schedule PDF");
    let pdf_bytes = state
        .client
        .get(TWENTY_EIGHT_X_SCHEDULE_URL)
        .send()
        .await
        .map_err(AppError::UpstreamError)?
        .bytes()
        .await
        .map_err(AppError::UpstreamError)?;

    let pdf_text = pdf_extract::extract_text_from_mem(&pdf_bytes)
        .map_err(|error| AppError::ScheduleError(error.to_string()))?;
    let schedule = parse_twenty_eight_x_schedule(&pdf_text)?;

    {
        let mut cache = state.twenty_eight_x_schedule_cache.lock().await;
        cache.data = Some(schedule.clone());
        cache.last_update = Some(Utc::now());
    }

    Ok(Json(schedule))
}

fn parse_twenty_eight_x_schedule(
    pdf_text: &str,
) -> Result<TwentyEightXScheduleResponse, AppError> {
    let weekday_rows = extract_schedule_rows(
        pdf_text,
        "MONDAY THROUGH FRIDAY SERVICE",
        "SUNDAY AND HOLIDAY SERVICE",
    )?;
    let sunday_rows = extract_schedule_rows(
        pdf_text,
        "SUNDAY AND HOLIDAY SERVICE",
        "SATURDAY SERVICE",
    )?;
    let saturday_rows = extract_schedule_rows(pdf_text, "SATURDAY SERVICE", "")?;

    let weekday_uc = parse_column_schedule(&weekday_rows, 5, 5 * 60 + 30)?;
    let weekday_tep = parse_column_schedule(&weekday_rows, 10, 15 * 60)?;
    let saturday_uc = parse_column_schedule(&saturday_rows, 5, 5 * 60 + 30)?;
    let saturday_tep = parse_column_schedule(&saturday_rows, 10, 15 * 60)?;
    let sunday_uc = parse_column_schedule(&sunday_rows, 5, 5 * 60 + 30)?;
    let sunday_tep = parse_column_schedule(&sunday_rows, 10, 15 * 60)?;

    let now = Local::now();
    let mut response = HashMap::new();

    if let Some(next_uc) = next_scheduled_arrivals(
        &now,
        &[
            (ServiceType::Weekday, &weekday_uc),
            (ServiceType::Saturday, &saturday_uc),
            (ServiceType::SundayHoliday, &sunday_uc),
        ],
        "Downtown / Oakland",
    ) {
        response.insert("7117".to_string(), next_uc);
    }

    if let Some(next_tep) = next_scheduled_arrivals(
        &now,
        &[
            (ServiceType::Weekday, &weekday_tep),
            (ServiceType::Saturday, &saturday_tep),
            (ServiceType::SundayHoliday, &sunday_tep),
        ],
        "Airport",
    ) {
        response.insert("4407".to_string(), next_tep);
    }

    Ok(response)
}

fn extract_schedule_rows(
    pdf_text: &str,
    start_marker: &str,
    end_marker: &str,
) -> Result<Vec<Vec<String>>, AppError> {
    let start = pdf_text
        .find(start_marker)
        .ok_or_else(|| AppError::ScheduleError(format!("Missing section: {start_marker}")))?;
    let end = if end_marker.is_empty() {
        pdf_text.len()
    } else {
        pdf_text[start..]
            .find(end_marker)
            .map(|offset| start + offset)
            .ok_or_else(|| AppError::ScheduleError(format!("Missing section: {end_marker}")))?
    };
    let section = &pdf_text[start..end];
    let time_regex = Regex::new(r"\b\d{1,2}:\d{2}\b")
        .map_err(|error| AppError::ScheduleError(error.to_string()))?;

    let rows = section
        .lines()
        .map(|line| {
            time_regex
                .find_iter(line)
                .map(|value| value.as_str().to_string())
                .collect::<Vec<_>>()
        })
        .filter(|times| times.len() >= 16)
        .collect::<Vec<_>>();

    if rows.is_empty() {
        return Err(AppError::ScheduleError(format!(
            "No timetable rows found in section: {start_marker}"
        )));
    }

    Ok(rows)
}

fn parse_column_schedule(
    rows: &[Vec<String>],
    column_index: usize,
    preferred_start_minutes: i64,
) -> Result<Vec<i64>, AppError> {
    let mut parsed_minutes = Vec::new();
    let mut last_value: Option<i64> = None;

    for row in rows {
        let raw_time = row.get(column_index).ok_or_else(|| {
            AppError::ScheduleError(format!("Missing column {column_index} in schedule row"))
        })?;
        let base_minutes = parse_clock_minutes(raw_time)?;
        let current = resolve_service_minutes(base_minutes, last_value, preferred_start_minutes)
            .ok_or_else(|| AppError::ScheduleError(format!("Could not resolve time {raw_time}")))?;
        parsed_minutes.push(current);
        last_value = Some(current);
    }

    Ok(parsed_minutes)
}

fn parse_clock_minutes(raw_time: &str) -> Result<i64, AppError> {
    let (hour_raw, minute_raw) = raw_time
        .split_once(':')
        .ok_or_else(|| AppError::ScheduleError(format!("Invalid time token: {raw_time}")))?;
    let hour = hour_raw
        .parse::<i64>()
        .map_err(|_| AppError::ScheduleError(format!("Invalid hour: {raw_time}")))?;
    let minute = minute_raw
        .parse::<i64>()
        .map_err(|_| AppError::ScheduleError(format!("Invalid minute: {raw_time}")))?;
    let normalized_hour = if hour == 12 { 12 } else { hour };
    Ok(normalized_hour * 60 + minute)
}

fn resolve_service_minutes(
    base_minutes: i64,
    last_value: Option<i64>,
    preferred_start_minutes: i64,
) -> Option<i64> {
    let candidates = [
        base_minutes,
        base_minutes + 12 * 60,
        base_minutes + 24 * 60,
        base_minutes + 36 * 60,
    ];

    if let Some(last) = last_value {
        candidates
            .into_iter()
            .find(|candidate| *candidate > last)
    } else {
        candidates
            .into_iter()
            .find(|candidate| *candidate >= preferred_start_minutes)
            .or_else(|| candidates.last().copied())
    }
}

fn next_scheduled_arrivals(
    now: &DateTime<Local>,
    schedules: &[(ServiceType, &Vec<i64>)],
    destination: &str,
) -> Option<Vec<ScheduledArrival>> {
    let mut upcoming = Vec::new();

    for day_offset in 0..8 {
        let service_date = now.date_naive() + Duration::days(day_offset);
        let service_type = service_type_for_weekday(service_date.weekday());

        let Some((_, minutes_list)) = schedules
            .iter()
            .find(|(candidate_service_type, _)| *candidate_service_type == service_type)
        else {
            continue;
        };

        for minutes_after_midnight in minutes_list.iter().copied() {
            let candidate_naive = service_date
                .and_hms_opt(0, 0, 0)?
                + Duration::minutes(minutes_after_midnight);
            let candidate = candidate_naive
                .and_local_timezone(Local)
                .single()?;
            if candidate > *now {
                upcoming.push(ScheduledArrival {
                    route: "28X".to_string(),
                    destination: destination.to_string(),
                    seconds: candidate.signed_duration_since(*now).num_seconds(),
                    scheduled: true,
                });
                if upcoming.len() == 3 {
                    return Some(upcoming);
                }
            }
        }
    }

    (!upcoming.is_empty()).then_some(upcoming)
}

fn service_type_for_weekday(weekday: Weekday) -> ServiceType {
    match weekday {
        Weekday::Sat => ServiceType::Saturday,
        Weekday::Sun => ServiceType::SundayHoliday,
        _ => ServiceType::Weekday,
    }
}
