#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use anyhow::{Ok, Result};
use dotenvy_macro::dotenv;
use prost::Message;
use reqwest::{
    header::{HeaderMap, ACCEPT, AUTHORIZATION},
    ClientBuilder, Method, Request,
};
use std::{fs::{create_dir, remove_dir_all, write}, io::{self, Result}};
use termcolor::{Color, ColorChoice, StandardStream};
use termcolor_output::{colored, colored_ln, termcolor_output_impl::ColoredOutput};
use transit_realtime::FeedEntity;

use crate::transit_realtime::FeedMessage;

mod transit_realtime {
    #![allow(clippy::all, clippy::pedantic, clippy::nursery, non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/transit_realtime.rs"));
}

#[tokio::main]
async fn main() -> Result<()> {
    const ENDPOINT: &str = "https://api.transport.nsw.gov.au/v1/gtfs/vehiclepos/buses";
    clear_output_directory()?;
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
    let message: FeedMessage = FeedMessage::decode(bytes)?;
    write("out/feed_message", format!("{:#?}", message))?;
    let entities: Vec<_> = message
        .entity
        .iter()
        .filter(|entity| {
            entity
                .vehicle
                .as_ref()
                .unwrap()
                .trip
                .as_ref()
                .unwrap()
                .route_id
                .as_ref()
                .unwrap()
                .ends_with("601")
        })
        .collect();
    write("out/routes_with_601", format!("{:#?}", entities))?;
    print_entities(&entities);
    Ok(())
}

fn clear_output_directory() -> Result<()> {
    remove_dir_all("out")?;
    create_dir("out")?;
    Ok(())
}

fn print_entities(entities: &Vec<&FeedEntity>) {
    let stdout = StandardStream::stdout(ColorChoice::Auto);
    for entity in entities {
        colored_ln(&mut stdout, |stdout| {
            colored!(
                stdout,
                "{}{}",
                fg!(Some(Color::Green)),
                format!("{:#?}", entity)
            )?; Result::Ok(())
        });
    }
}
