mod models;

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::Utc;
use hmac::{Hmac, Mac};
use lambda_http::http::StatusCode;
use lambda_http::{run, service_fn, Error, IntoResponse, Request, Response};
use serde_json::json;
use sha3::Sha3_256;
use tracing::{debug, error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

use crate::models::{MomentoModel, MomentoPayload};

type HmacSha3_256 = Hmac<Sha3_256>;

/// answers the question as to whether the received payloads publish time is within the
/// second limit of being "fresh" enough.  Anything older than 60 seconds will be marked as rejected
fn is_request_new_enough(published: i64) -> bool {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    // let time_since_published = since_the_epoch.as_millis() - published as u128;
    let new_duration = Duration::from_millis(published as u64);
    let calculated = since_the_epoch - new_duration;

    debug!(
        since_the_epoch = since_the_epoch.as_millis(),
        published = published,
        time_since_published = calculated.as_secs(),
        "Time since published"
    );
    calculated.as_secs() < 60
}

/// Main function handler.  This is executed when a request comes in via the Function URL
async fn function_handler(secret_string: &str, event: Request) -> Result<impl IntoResponse, Error> {
    let mut status_code = StatusCode::OK;
    let mut return_body = "Success";

    // extra and parse the body of the request into a MomentoPayload
    let body = event.body();
    let body_string = std::str::from_utf8(body).expect("Body wasn't supplied");
    let payload: Result<MomentoPayload, serde_json::Error> = serde_json::from_str(body_string);
    let header_value = event.headers().get("momento-signature");
    // header value is required to compare against the HMAC of the incoming payload
    match header_value {
        Some(header_value) => match payload {
            Ok(payload) => {
                let new_enough = is_request_new_enough(payload.publish_timestamp);
                if !new_enough {
                    error!(
                        "Request is not new enough.  Publish time is {}",
                        payload.publish_timestamp
                    );
                    status_code = StatusCode::FORBIDDEN;
                    return_body = "Unauthorized";
                }

                let verified =
                    verify_signature(&payload, secret_string, header_value.to_str().unwrap());
                if verified {
                    let mut model: MomentoModel = serde_json::from_str(&payload.text).unwrap();
                    model.received_timestamp = Utc::now();
                    info!(
                        "(Model)={model:?}|(TimeBetweenMoPublishedAndReceived)={}|(TimeBetweenPublishedAndReceived)={}",
                        model.time_between_mo_publish_and_received(payload.publish_timestamp),
                        model.time_between_publish_and_received()
                    );
                } else {
                    error!("Signature verification failed");
                    status_code = StatusCode::FORBIDDEN;
                    return_body = "Unauthorized";
                }
            }
            Err(_e) => {
                error!("Error parsing Momento payload");
                status_code = StatusCode::FORBIDDEN;
                return_body = "Unauthorized";
            }
        },
        None => {
            error!("Header momento-signature is missing");
            status_code = StatusCode::FORBIDDEN;
            return_body = "Unauthorized";
        }
    }

    // the final output response sent back to the client
    let response = Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(
            json!({
              "message": return_body,
            })
            .to_string(),
        )
        .map_err(Box::new)?;

    info!(body = return_body, "Output of request={:?}", status_code);
    Ok(response)
}

/// Verifies the signature of the incoming payload against the secret string.  Uses SHA3-256 to HMAC the incoming payload
/// and compare that against the value of the Header momento-signature that MUST be present in the request
fn verify_signature(payload: &MomentoPayload, secret_string: &str, signature: &str) -> bool {
    let s = serde_json::to_string(&payload).expect("Error serde");
    let mac3 = HmacSha3_256::new_from_slice(secret_string.as_bytes());
    match mac3 {
        Ok(mut m) => {
            m.update(s.as_ref());
            let result3 = m.finalize();
            let code_bytes_3 = result3.into_bytes();

            hex::encode(code_bytes_3) == signature
        }
        Err(_) => false,
    }
}

/// Main function which is the starting point for the Lambda
#[tokio::main]
async fn main() -> Result<(), Error> {
    let filtered_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .json()
        .with_target(true)
        .with_file(true);

    tracing_subscriber::registry()
        .with(filtered_layer)
        .with(EnvFilter::from_default_env())
        .init();

    let config = aws_config::load_from_env().await;
    let secrets_client = aws_sdk_secretsmanager::Client::new(&config);

    let resp = secrets_client
        .get_secret_value()
        .secret_id("moment-webhook-token")
        .send()
        .await?;
    let string_field = resp
        .secret_string()
        .expect("Secret string must have a value");
    run(service_fn(move |payload: Request| async move {
        function_handler(string_field, payload).await
    }))
    .await
}
