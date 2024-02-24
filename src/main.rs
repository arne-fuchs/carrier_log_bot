use std::ops::Add;
use std::{process, thread};
use std::thread::JoinHandle;
use std::time::Duration;
use base64::Engine;
use base64::engine::general_purpose;
use chrono::{DateTime, Utc};
use discord::{Connection, Discord};
use discord::model::ChannelId;
use iota_sdk::packable::PackableExt;
use iota_sdk::types::block::{Block, ConvertTo};
use iota_sdk::types::block::payload::Payload;
use iota_sdk::types::block::signature::Ed25519Signature;
use rustc_hex::FromHex;
use tonic::codegen::tokio_stream::StreamExt;
use tonic::transport::{Channel, Uri};
mod journal_reader;

use crate::inx_handler::proto;
pub use crate::inx_handler::proto::inx_client;
mod inx_handler;

#[tokio::main]
async fn main() {
    let inx_address = std::env::var("INX_ADDRESS").unwrap();
    let discord = Discord::from_bot_token(std::env::var("BOT_TOKEN").unwrap().as_str()).expect("Login failed");
    let (connection, _) = discord.connect().expect("Connect failed");
    connection.set_game_name("Cruising in the void".to_string());
    let channel = ChannelId(std::env::var("CHANNEL_ID").unwrap().parse().unwrap());

    println!("Connecting to inx...");

    let inx_url: Uri = {
        let mut string = String::from("http://");
        string.push_str(inx_address.as_str());
        string.clone().as_str()
    }.parse().unwrap();

    let inx_channel = {
        let mut result = Channel::builder(inx_url.clone()).connect().await;
        while result.is_err(){
            println!("Trying to connect to inx... ({})({})", &inx_url,&result.err().unwrap());
            tokio::time::sleep(Duration::from_secs(5)).await;
            result = Channel::builder(inx_url.clone()).connect().await;
        }

        result.unwrap()
    };
    let mut inx_client = inx_client::InxClient::new(inx_channel);

    let mut response_node_status = inx_client.read_node_status(
        proto::NoParams{}
    ).await.expect("Request failed");

    let mut node_status = response_node_status.into_inner();
    while !node_status.is_healthy && !node_status.is_synced {
        println!("Waiting for node to be healthy and synced...");
        println!("Health: {}\t Synced: {}", &node_status.is_healthy,&node_status.is_synced);
        tokio::time::sleep(Duration::from_secs(5)).await;
        response_node_status = inx_client.read_node_status(
            proto::NoParams{}
        ).await.expect("Failed requesting node status");

        node_status = response_node_status.into_inner();
    }

    //Node is synced and healthy at this point
    println!("Connected and healthy!");

    let response_listen_blocks = inx_client.listen_to_blocks(
        proto::NoParams{}
    ).await.expect("Failed listening to blocks");
    let mut block_stream = response_listen_blocks.into_inner();
    let mut handle: Option<JoinHandle<()>> = None;
    loop {
        let stream_block = block_stream.next().await;
        match stream_block {
            None => {
                println!("Couldn't find block");
            }
            Some(block_result) => {
                match block_result {
                    Ok(proto_block) => {
                        //println!("0x{}",hex::encode(block.block_id.unwrap().id));
                        match proto_block.block {
                            None => {
                                eprintln!("No raw block found");
                            }
                            Some(raw_block) => {
                                let block_unpack_result = Block::unpack_unverified(raw_block.data);
                                match block_unpack_result {
                                    Ok(block) => {
                                        match block.payload() {
                                            None => {
                                                eprintln!("Couldn't found payload for block")
                                            }
                                            Some(payload) => {
                                                let payload = payload.clone();

                                                    match payload {
                                                        Payload::Transaction(_) => {}
                                                        Payload::Milestone(_) => {}
                                                        Payload::TreasuryTransaction(_) => {}
                                                        Payload::TaggedData(tagged_data) => {
                                                            let result = json::parse(String::from_utf8(tagged_data.data().to_vec()).unwrap().as_str());
                                                            if let Ok(json) = result {
                                                                //println!("{}",&json);

                                                                let data = general_purpose::STANDARD.decode(json["message"].as_str().unwrap()).unwrap();

                                                                let p_key = json["public_key"].to_string();
                                                                let pub_key_bytes: Vec<u8> = json["public_key"].as_str().unwrap()[2..].from_hex().unwrap();
                                                                let mut pub_key: [u8;32] = [0u8;32];
                                                                pub_key[0..32].copy_from_slice(&pub_key_bytes[0..32]);

                                                                let sig_bytes: Vec<u8> = json["signature"].as_str().unwrap()[2..].from_hex().unwrap();
                                                                let mut sig: [u8;64] = [0u8;64];
                                                                sig[0..64].copy_from_slice(&sig_bytes[0..64]);

                                                                let sig = Ed25519Signature::try_from_bytes(pub_key,sig).unwrap();

                                                                if sig.verify(data.as_slice()) {
                                                                    //Data is verified -> You can work with it
                                                                    let event = json["event"].as_str().unwrap();
                                                                    match event {
                                                                        //{ "timestamp":"2022-10-13T10:01:35Z", "event":"CarrierStats", "CarrierID":3704402432, "Callsign":"Q2K-BHB", "Name":"FUXBAU", "DockingAccess":"squadron", "AllowNotorious":true, "FuelLevel":617, "JumpRangeCurr":500.000000, "JumpRangeMax":500.000000, "PendingDecommission":false, "SpaceUsage":{ "TotalCapacity":25000, "Crew":6170, "Cargo":9331, "CargoSpaceReserved":1272, "ShipPacks":0, "ModulePacks":433, "FreeSpace":7794 }, "Finance":{ "CarrierBalance":1184935299, "ReserveBalance":51460958, "AvailableBalance":1029659181, "ReservePercent":4, "TaxRate_shipyard":15, "TaxRate_rearm":100, "TaxRate_outfitting":15, "TaxRate_refuel":100, "TaxRate_repair":100 }, "Crew":[ { "CrewRole":"BlackMarket", "Activated":false }, { "CrewRole":"Captain", "Activated":true, "Enabled":true, "CrewName":"Vada Cannon" }, { "CrewRole":"Refuel", "Activated":true, "Enabled":true, "CrewName":"Donna Moon" }, { "CrewRole":"Repair", "Activated":true, "Enabled":true, "CrewName":"Darnell Grant" }, { "CrewRole":"Rearm", "Activated":true, "Enabled":true, "CrewName":"Eiza York" }, { "CrewRole":"Commodities", "Activated":true, "Enabled":true, "CrewName":"Jewel King" }, { "CrewRole":"VoucherRedemption", "Activated":true, "Enabled":true, "CrewName":"Ezra Ramirez" }, { "CrewRole":"Exploration", "Activated":true, "Enabled":true, "CrewName":"Kasey Callahan" }, { "CrewRole":"Shipyard", "Activated":true, "Enabled":true, "CrewName":"Abby Cooke" }, { "CrewRole":"Outfitting", "Activated":true, "Enabled":true, "CrewName":"Jayne Callahan" }, { "CrewRole":"CarrierFuel", "Activated":true, "Enabled":true, "CrewName":"Abraham Strickland" }, { "CrewRole":"VistaGenomics", "Activated":true, "Enabled":true, "CrewName":"Melinda Reilly" }, { "CrewRole":"PioneerSupplies", "Activated":false }, { "CrewRole":"Bartender", "Activated":true, "Enabled":true, "CrewName":"Dean Barlow" } ], "ShipPacks":[  ], "ModulePacks":[ { "PackTheme":"VehicleSupport", "PackTier":1 }, { "PackTheme":"Storage", "PackTier":2 } ] }

                                                                        "CarrierStats" => {
                                                                            println!("CarrierStats");
                                                                            let name = json["Name"].to_string().add(" ").add(json["Callsign"].as_str().unwrap());
                                                                            connection.set_game_name(name);
                                                                        }
                                                                        //{ "timestamp":"2022-11-29T21:09:30Z", "event":"CarrierJumpRequest", "CarrierID":3704402432, "SystemName":"Ngorowai", "Body":"Ngorowai A", "SystemAddress":4207155286722, "BodyID":1, "DepartureTime":"2022-11-29T21:24:40Z" }
                                                                        "CarrierJumpRequest" => {
                                                                            println!("CarrierJumpRequest");
                                                                            let text = format!("__**JUMP INITIATED**__\nDestination: {}\nBody: {}\nDeparture: {}",json["SystemName"],json["Body"],json["DepartureTime"]);
                                                                            discord.send_message(channel,text.as_str(),"",false).unwrap();
                                                                            match DateTime::parse_from_rfc3339(json["DepartureTime"].as_str().unwrap()) {
                                                                                Ok(target_time) => {
                                                                                    let now = Utc::now();
                                                                                    let time_difference = target_time.signed_duration_since(now);
                                                                                    if time_difference.is_zero() {
                                                                                        println!("The time is in the past");
                                                                                    }else {
                                                                                        let sleep_duration = time_difference.to_std().unwrap().add(Duration::from_secs(300));

                                                                                        handle = Some(thread::spawn(move || {
                                                                                            thread::sleep(sleep_duration);
                                                                                        }));
                                                                                    }
                                                                                }
                                                                                Err(err) => {
                                                                                    eprintln!("Error parsing the time format: {}", err);
                                                                                }
                                                                            };
                                                                        }
                                                                        "CarrierTradeOrder" => {}
                                                                        "CarrierFinance" => {}
                                                                        //{ "timestamp":"2022-08-19T17:15:07Z", "event":"CarrierJumpCancelled", "CarrierID":3704402432 }
                                                                        "CarrierJumpCancelled" => {
                                                                            println!("CarrierJumpCancelled:");
                                                                            let text = format!("__**JUMP CANCELED**__");
                                                                            discord.send_message(channel,text.as_str(),"",false).unwrap();
                                                                            handle = None;
                                                                        }
                                                                        "CarrierDepositFuel" => {}
                                                                        "CarrierDockingPermission" => {}
                                                                        "CarrierCrewServices" => {}
                                                                        "Shutdown" => {}
                                                                        _ => {}
                                                                    }
                                                                } else {
                                                                    println!("Signature verification failed.");
                                                                }
                                                            }
                                                        }
                                                    }

                                            }
                                        }
                                    }
                                    Err(err) => {
                                        eprintln!("Unpacking raw block failed: {}", err);
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("Error getting block: {}", err);
                    }
                }
            }
        }
    }
}