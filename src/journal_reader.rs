use std::{fs, thread};
use std::fs::{DirEntry, File};
use std::io::{BufRead, BufReader};
use std::ops::Add;
use std::str::FromStr;
use std::thread::JoinHandle;
use std::time::Duration;
use chrono::{DateTime, NaiveDateTime, Utc};
use discord::{Connection, Discord};
use discord::model::{ChannelId, UserId};

pub struct JournalReader{
    pub reader: BufReader<File>,
    discord: Discord,
    connection: Connection,
    channel: ChannelId,
    handle: Option<JoinHandle<()>>,
    status: String,
}

pub fn initialize(directory_path: String,discord: Discord, connection: Connection, channel: ChannelId) -> JournalReader{
    let reader = get_journal_log_by_index(directory_path.clone(),0);
    JournalReader{
        reader,
        discord,
        connection,
        channel,
        handle: None,
        status: "financial ruin".to_string()
    }
}

impl JournalReader {

    pub fn run(&mut self){
        let mut line = String::new();

        if self.handle.is_some() && self.handle.as_ref().unwrap().is_finished() {
            let admin_channel_id = self.discord.create_dm(UserId(u64::from_str(std::env::var("ADMIN_ID").unwrap().as_str()).unwrap())).unwrap().id;
            self.discord.send_message( admin_channel_id,"Carrier is ready to jump!","",false).unwrap();
            self.handle = None;
            self.connection = self.discord.connect().expect("Couldn't connect.").0;
            self.connection.set_game_name(self.status.clone());
        }

        match self.reader.read_line(&mut line) {
            Ok(flag) => {
                if flag == 0 {
                    //Reached EOF -> does not mean new data wont come in
                }else {
                    if !line.eq("") {
                        let json_result = json::parse(&line);
                        match json_result {
                            Ok(json) => {
                                let event = json["event"].as_str().unwrap();
                                match event {
                                    //{ "timestamp":"2022-10-13T10:01:35Z", "event":"CarrierStats", "CarrierID":3704402432, "Callsign":"Q2K-BHB", "Name":"FUXBAU", "DockingAccess":"squadron", "AllowNotorious":true, "FuelLevel":617, "JumpRangeCurr":500.000000, "JumpRangeMax":500.000000, "PendingDecommission":false, "SpaceUsage":{ "TotalCapacity":25000, "Crew":6170, "Cargo":9331, "CargoSpaceReserved":1272, "ShipPacks":0, "ModulePacks":433, "FreeSpace":7794 }, "Finance":{ "CarrierBalance":1184935299, "ReserveBalance":51460958, "AvailableBalance":1029659181, "ReservePercent":4, "TaxRate_shipyard":15, "TaxRate_rearm":100, "TaxRate_outfitting":15, "TaxRate_refuel":100, "TaxRate_repair":100 }, "Crew":[ { "CrewRole":"BlackMarket", "Activated":false }, { "CrewRole":"Captain", "Activated":true, "Enabled":true, "CrewName":"Vada Cannon" }, { "CrewRole":"Refuel", "Activated":true, "Enabled":true, "CrewName":"Donna Moon" }, { "CrewRole":"Repair", "Activated":true, "Enabled":true, "CrewName":"Darnell Grant" }, { "CrewRole":"Rearm", "Activated":true, "Enabled":true, "CrewName":"Eiza York" }, { "CrewRole":"Commodities", "Activated":true, "Enabled":true, "CrewName":"Jewel King" }, { "CrewRole":"VoucherRedemption", "Activated":true, "Enabled":true, "CrewName":"Ezra Ramirez" }, { "CrewRole":"Exploration", "Activated":true, "Enabled":true, "CrewName":"Kasey Callahan" }, { "CrewRole":"Shipyard", "Activated":true, "Enabled":true, "CrewName":"Abby Cooke" }, { "CrewRole":"Outfitting", "Activated":true, "Enabled":true, "CrewName":"Jayne Callahan" }, { "CrewRole":"CarrierFuel", "Activated":true, "Enabled":true, "CrewName":"Abraham Strickland" }, { "CrewRole":"VistaGenomics", "Activated":true, "Enabled":true, "CrewName":"Melinda Reilly" }, { "CrewRole":"PioneerSupplies", "Activated":false }, { "CrewRole":"Bartender", "Activated":true, "Enabled":true, "CrewName":"Dean Barlow" } ], "ShipPacks":[  ], "ModulePacks":[ { "PackTheme":"VehicleSupport", "PackTier":1 }, { "PackTheme":"Storage", "PackTier":2 } ] }
                                    "CarrierStats" => {
                                        println!("CarrierStats: {}",line);
                                        let name = json["Name"].to_string().add(" ").add(json["Callsign"].as_str().unwrap());
                                        self.status = name.clone();
                                        self.connection.set_game_name(name);
                                    }
                                    //{ "timestamp":"2022-11-29T21:09:30Z", "event":"CarrierJumpRequest", "CarrierID":3704402432, "SystemName":"Ngorowai", "Body":"Ngorowai A", "SystemAddress":4207155286722, "BodyID":1, "DepartureTime":"2022-11-29T21:24:40Z" }
                                    "CarrierJumpRequest" => {
                                        println!("CarrierJumpRequest: {}",line);
                                        let text = format!("__**JUMP INITIATED**__\nDestination: {}\nBody: {}\nDeparture: {}",json["SystemName"],json["Body"],json["DepartureTime"]);
                                        self.discord.send_message(self.channel,text.as_str(),"",false).unwrap();
                                        let target_time = match DateTime::parse_from_rfc3339(json["DepartureTime"].as_str().unwrap()) {
                                            Ok(time) => time,
                                            Err(err) => {
                                                eprintln!("Error parsing the time format: {}", err);
                                                return;
                                            }
                                        };
                                        let now = Utc::now();
                                        let time_difference = target_time.signed_duration_since(now);
                                        if time_difference.is_zero() {
                                            println!("The time is in the past");
                                            return;
                                        }
                                        let sleep_duration = time_difference.to_std().unwrap().add(Duration::from_secs(300));
                                        let handle = thread::spawn(move || {
                                            thread::sleep(sleep_duration);
                                        });

                                        self.handle = Some(handle);
                                    }
                                    "CarrierTradeOrder" => {}
                                    "CarrierFinance" => {}
                                    //{ "timestamp":"2022-08-19T17:15:07Z", "event":"CarrierJumpCancelled", "CarrierID":3704402432 }
                                    "CarrierJumpCancelled" => {
                                        println!("CarrierJumpCancelled: {}",line);
                                        let text = format!("__**JUMP CANCELED**__");
                                        self.discord.send_message(self.channel,text.as_str(),"",false).unwrap();
                                        self.handle = None;
                                    }
                                    "CarrierDepositFuel" => {}
                                    "CarrierDockingPermission" => {}
                                    "CarrierCrewServices" => {}
                                    "Shutdown" => {
                                        std::process::exit(0);
                                    }
                                    _ => {}
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }
                line.clear();
            }
            Err(_err) => {
                println!("Error reading journal file!");
            }
        };
    }
}

fn get_journal_log_by_index(mut directory_path: String, index: usize) -> BufReader<File> {
    let directory = fs::read_dir(directory_path.clone()).unwrap();

    let mut log_name_date_list: Vec<String> = Vec::new();

    for file in directory {
        let dir_entry: DirEntry = file.unwrap();
        let file_name: String = dir_entry.file_name().into_string().unwrap().to_owned();
        let split_file_name = file_name.split(".");
        let name_parts: Vec<&str> = split_file_name.collect::<Vec<&str>>();

        if name_parts[&name_parts.len()-1] == "log" {
            log_name_date_list.push(name_parts[1].to_owned());
        }
    }

    let date_string_format = "%Y-%m-%dT%H%M%S";
    log_name_date_list.sort_by(|a,b|{
        let date_time_a = NaiveDateTime::parse_from_str(a, date_string_format).unwrap_or_default();
        let date_time_b = NaiveDateTime::parse_from_str(b, date_string_format).unwrap_or_default();

        date_time_a.cmp(&date_time_b).reverse()
    });

    //Reader WILL crash at this point by an index out of bounds exception if it cant find any more logs
    directory_path.push_str("/Journal.");
    directory_path.push_str(log_name_date_list[index].to_owned().as_str());
    directory_path.push_str(".01.log");

    println!("Opening journal: {}",directory_path);

    let journal_log_file = File::open(&directory_path).expect("file not found!");

    BufReader::new(journal_log_file)
}