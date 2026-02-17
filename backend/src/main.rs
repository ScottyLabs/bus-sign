use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use quick_xml::de::from_str;
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};

const DEFAULT_URL: &str = "https://truetime.portauthority.org/bustime/eta/getStopPredictionsETA.jsp?agency=&route=all&stop=Port%20Authority%20Bus%3A4407&key=0.6308738799301408";
const STOP_7117_URL: &str = "https://truetime.portauthority.org/bustime/eta/getStopPredictionsETA.jsp?agency=&route=all&stop=Port%20Authority%20Bus%3A7117&key=0.3685356195902162";

#[derive(Debug, Deserialize)]
#[serde(rename = "stop")]
struct Stop {
    #[serde(rename = "pre", default)]
    predictions: Vec<Prediction>,
}

#[derive(Debug, Deserialize)]
struct Prediction {
    #[serde(rename = "pt")]
    minutes: Option<String>,
    #[serde(rename = "pu")]
    units: Option<String>,
    #[serde(rename = "fd")]
    destination: Option<String>,
    #[serde(rename = "rn")]
    route_short: Option<String>,
    #[serde(rename = "rd")]
    route: Option<String>,
    #[serde(rename = "nextbusonroutetime")]
    on_route_time: Option<String>,
    #[serde(rename = "v")]
    vehicle_id: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct OutputPrediction {
    #[serde(rename = "ETA")]
    eta: Option<String>,
    #[serde(rename = "location")]
    location: Option<String>,
    #[serde(rename = "route")]
    route: Option<String>,
    #[serde(rename = "busNumber")]
    bus_number: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StopQuery {
    stop: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct StopResponse {
    stop: String,
    predictions: Vec<OutputPrediction>,
}

#[derive(Debug, Deserialize)]
struct PredictionsQuery {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FetchQuery {
    url: Option<String>,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/predictions", get(get_predictions))
        .route("/fetch", get(fetch_raw))
        .route("/stop", get(get_stop))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8787").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_predictions(
    Query(query): Query<PredictionsQuery>,
) -> Result<Json<Vec<OutputPrediction>>, ApiError> {
    let url = query.url.unwrap_or_else(|| DEFAULT_URL.to_string());
    let body = reqwest::get(url)
        .await
        .map_err(|err| ApiError {
            status: StatusCode::BAD_GATEWAY,
            message: format!("request failed: {err}"),
        })?
        .text()
        .await
        .map_err(|err| ApiError {
            status: StatusCode::BAD_GATEWAY,
            message: format!("read body failed: {err}"),
        })?;

    let stop: Stop = from_str(&body).map_err(|err| ApiError {
        status: StatusCode::BAD_GATEWAY,
        message: format!("parse failed: {err}"),
    })?;
    Ok(Json(to_output_predictions(stop.predictions)))
}

async fn fetch_raw(Query(query): Query<FetchQuery>) -> Result<Response, ApiError> {
    let url = query.url.ok_or_else(|| ApiError {
        status: StatusCode::BAD_REQUEST,
        message: "missing url query parameter".to_string(),
    })?;

    let body = reqwest::get(url)
        .await
        .map_err(|err| ApiError {
            status: StatusCode::BAD_GATEWAY,
            message: format!("request failed: {err}"),
        })?
        .text()
        .await
        .map_err(|err| ApiError {
            status: StatusCode::BAD_GATEWAY,
            message: format!("read body failed: {err}"),
        })?;

    Ok((StatusCode::OK, body).into_response())
}

async fn get_stop(Query(query): Query<StopQuery>) -> Result<Json<StopResponse>, ApiError> {
    let stop = query.stop.unwrap_or_else(|| "4407".to_string());

    let url = match stop.as_str() {
        "4407" => DEFAULT_URL,
        "7117" => STOP_7117_URL,
        _ => {
            return Err(ApiError {
                status: StatusCode::BAD_REQUEST,
                message: "unsupported stop; use 4407 or 7117".to_string(),
            })
        }
    };

    let body = reqwest::get(url)
        .await
        .map_err(|err| ApiError {
            status: StatusCode::BAD_GATEWAY,
            message: format!("request failed: {err}"),
        })?
        .text()
        .await
        .map_err(|err| ApiError {
            status: StatusCode::BAD_GATEWAY,
            message: format!("read body failed: {err}"),
        })?;

    let stop_data: Stop = from_str(&body).map_err(|err| ApiError {
        status: StatusCode::BAD_GATEWAY,
        message: format!("parse failed: {err}"),
    })?;

    Ok(Json(StopResponse {
        stop,
        predictions: to_output_predictions(stop_data.predictions),
    }))
}

fn to_output_predictions(predictions: Vec<Prediction>) -> Vec<OutputPrediction> {
    predictions
        .into_iter()
        .map(|prediction| OutputPrediction {
            eta: prediction.minutes,
            location: prediction.destination,
            route: prediction.route_short,
            bus_number: prediction.vehicle_id,
        })
        .collect()
}
