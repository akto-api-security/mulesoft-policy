// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};
use chrono::Utc;
use pdk::hl::*;
use pdk::logger::debug;
use serde_json::json;
use std::time::Duration;
use std::collections::HashMap;
use crate::generated::config::Config;
use rand::Rng;
use pdk::logger::info;

#[derive(Clone)]
struct ReqPayload {
    path: String,
    method: String,
    request_headers: String,
    request_body: String,
}

async fn process_request(state: RequestState) -> Flow<ReqPayload> {
    let headers_state = state.into_headers_state().await;

    let raw_headers = headers_state.handler().headers();

    let mut headers_map = HashMap::new();
    for (key, value) in raw_headers {
        headers_map.insert(key.to_string(), value.to_string());
    }
    let host = headers_map.get("host")
    .or_else(|| headers_map.get(":authority"))
    .map(|h| h.clone())
    .unwrap_or_else(|| "".to_string());

    if !headers_map.contains_key("host") {
        if let Some(authority) = headers_map.get(":authority") {
            headers_map.insert("host".to_string(), authority.clone());

        }
    }

    let request_headers_json = serde_json::to_string(&headers_map).unwrap_or_default();
    
    let method = headers_state.method();
    let path = headers_state.path();
    let body_state = headers_state.into_body_state().await;
    let request_body = body_state.handler().body();
    let request_payload = String::from_utf8_lossy(&request_body).to_string();

    let full_url = format!("{}{}", host, path);

    Flow::Continue(ReqPayload {
        path: full_url,
        method,
        request_headers: request_headers_json,
        request_body: request_payload,
    })
}

async fn response_filter(
    state: ResponseState, config: &Config, client: &HttpClient, request_data: RequestData<ReqPayload>
) {

    if let RequestData::Continue(req_payload) = request_data{

        let sample_percentage = config.sampling_percentage.unwrap_or(100);
        let random_value: u8 = rand::thread_rng().gen_range(1..=100);
        if i64::from(random_value) > sample_percentage {
            debug!("Skipping this api: {:?}", random_value);
            return;
        }
        let response_headers_state = state.into_headers_state().await;
        let raw_headers = response_headers_state.handler().headers();

        // Convert the header array into a HashMap
        let mut headers_map = HashMap::new();
        for (key, value) in raw_headers {
            headers_map.insert(key.to_string(), value.to_string());
        }

        let response_headers_json = serde_json::to_string(&headers_map).unwrap_or_default();
        let response_status_code = response_headers_state.status_code();
        let body_state = response_headers_state.into_body_state().await;
        let body_handler = body_state.handler();
        let body = body_handler.body();
        let response_payload = String::from_utf8_lossy(&body).to_string();
   
        // Build the data batch in the required format
        let batch_data = json!({
            "batchData": [
                {
                    "path": req_payload.path,
                    "requestHeaders": req_payload.request_headers,
                    "responseHeaders": response_headers_json,
                    "method": req_payload.method,
                    "requestPayload": req_payload.request_body,
                    "responsePayload": response_payload,
                    "ip": "0.0.0.0",  // Placeholder IP, replace with real client IP if available
                    "time": Utc::now().timestamp(),
                    "statusCode": response_status_code,
                    "type": "HTTP/1.1",
                    "status": "",
                    "akto_account_id": "1000000",
                    "akto_vxlan_id": "123",
                    "is_pending": "false",
                    "source": "MIRRORING"
                }
            ]
        });

        // Log batch data for debugging
        debug!("Batch data: {:?}", batch_data);

        let payload = batch_data.to_string();
        let response = client
            .request(&config.ingestion_url)
            .timeout(Duration::from_secs(10))
            .body(payload.as_bytes())
            .headers(vec![("content-type", "application/json")])
            .post()
            .await;

        match response {
            Ok(resp) if [200, 202, 204].contains(&resp.status_code()) => {
                debug!("Post call successful: {:?}", resp);
                //Continue(())
            }
            Ok(resp) => {
                debug!("Post call failed with status: {:?}", resp.status_code());
                //Break(Response::new(500).with_body("Failed to send batch data"))
            }
            Err(err) => {
                debug!("Error during post call: {:?}", err);
                //Break(Response::new(500).with_body("Internal server error"))
            }
        }
    };
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(config_bytes): Configuration, client: HttpClient) -> Result<()> {

    let config: Config = serde_json::from_slice(&config_bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&config_bytes),
            err
        )
    })?;

    let filter = on_request(|rs| process_request(rs)).on_response(|rs, request_data| {
        response_filter(rs, &config, &client, request_data)
    });
    launcher.launch(
            filter
        ).await?;
    Ok(())
}