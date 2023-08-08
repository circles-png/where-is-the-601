use anyhow::{Ok, Result};
use dotenvy_macro::dotenv;
use prost::Message;
use reqwest::{
    header::{HeaderMap, ACCEPT, AUTHORIZATION},
    ClientBuilder, Method, Request,
};
use std::fs::{write, remove_dir_all, create_dir};

use crate::transit_realtime::FeedMessage;

mod transit_realtime {
    #![allow(non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/transit_realtime.rs"));
}

#[tokio::main]
async fn main() -> Result<()> {
    const ENDPOINT: &str = "https://api.transport.nsw.gov.au/v1/gtfs/vehiclepos/buses";
    remove_dir_all("out")?;
    create_dir("out")?;
    let bytes = ClientBuilder::new()
        .default_headers({
            let mut header_map = HeaderMap::new();
            header_map.append(ACCEPT, "application/x-google-protobuf".parse()?);
            header_map.append(
                AUTHORIZATION,
                format!("apikey {}", dotenv!("API_KEY")).parse()?,
            );
            header_map
        })
        .build()?
        .execute(Request::new(Method::GET, ENDPOINT.parse()?))
        .await?
        .bytes()
        .await?;
    write("out/data", bytes.clone())?;
    let message = FeedMessage::decode(bytes)?;
    write("out/feed_message", format!("{:#?}", message))?;

    Ok(())
}
