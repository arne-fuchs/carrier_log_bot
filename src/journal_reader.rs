use std::fmt::format;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::{BufRead, BufReader};
use chrono::{NaiveDate, NaiveDateTime};
use discord::Discord;
use discord::model::ChannelId;
use json::JsonValue;

pub struct JournalReader{
    pub reader: BufReader<File>,
    directory_path: String,
    discord: Discord,
    channel: ChannelId
}

pub fn initialize(directory_path: String,discord: Discord, channel: ChannelId) -> JournalReader{
    let reader = get_journal_log_by_index(directory_path.clone(),0);
    JournalReader{
        reader,
        directory_path,
        discord,
        channel
    }
}

impl JournalReader {

    pub fn run(&mut self){
        let mut line = String::new();

        match self.reader.read_line(&mut line) {
            Ok(flag) => {
                if flag == 0 {
                    //Reached EOF -> does not mean new data wont come in
                }else {
                    if !line.eq("") {
                        //TODO Here logic what do write to discord
                        let json = json::parse(&line).unwrap();
                        let event = json["event"].as_str().unwrap();
                        match event {
                            //{ "timestamp":"2022-10-13T10:01:35Z", "event":"CarrierStats", "CarrierID":3704402432, "Callsign":"Q2K-BHB", "Name":"FUXBAU", "DockingAccess":"squadron", "AllowNotorious":true, "FuelLevel":617, "JumpRangeCurr":500.000000, "JumpRangeMax":500.000000, "PendingDecommission":false, "SpaceUsage":{ "TotalCapacity":25000, "Crew":6170, "Cargo":9331, "CargoSpaceReserved":1272, "ShipPacks":0, "ModulePacks":433, "FreeSpace":7794 }, "Finance":{ "CarrierBalance":1184935299, "ReserveBalance":51460958, "AvailableBalance":1029659181, "ReservePercent":4, "TaxRate_shipyard":15, "TaxRate_rearm":100, "TaxRate_outfitting":15, "TaxRate_refuel":100, "TaxRate_repair":100 }, "Crew":[ { "CrewRole":"BlackMarket", "Activated":false }, { "CrewRole":"Captain", "Activated":true, "Enabled":true, "CrewName":"Vada Cannon" }, { "CrewRole":"Refuel", "Activated":true, "Enabled":true, "CrewName":"Donna Moon" }, { "CrewRole":"Repair", "Activated":true, "Enabled":true, "CrewName":"Darnell Grant" }, { "CrewRole":"Rearm", "Activated":true, "Enabled":true, "CrewName":"Eiza York" }, { "CrewRole":"Commodities", "Activated":true, "Enabled":true, "CrewName":"Jewel King" }, { "CrewRole":"VoucherRedemption", "Activated":true, "Enabled":true, "CrewName":"Ezra Ramirez" }, { "CrewRole":"Exploration", "Activated":true, "Enabled":true, "CrewName":"Kasey Callahan" }, { "CrewRole":"Shipyard", "Activated":true, "Enabled":true, "CrewName":"Abby Cooke" }, { "CrewRole":"Outfitting", "Activated":true, "Enabled":true, "CrewName":"Jayne Callahan" }, { "CrewRole":"CarrierFuel", "Activated":true, "Enabled":true, "CrewName":"Abraham Strickland" }, { "CrewRole":"VistaGenomics", "Activated":true, "Enabled":true, "CrewName":"Melinda Reilly" }, { "CrewRole":"PioneerSupplies", "Activated":false }, { "CrewRole":"Bartender", "Activated":true, "Enabled":true, "CrewName":"Dean Barlow" } ], "ShipPacks":[  ], "ModulePacks":[ { "PackTheme":"VehicleSupport", "PackTier":1 }, { "PackTheme":"Storage", "PackTier":2 } ] }
                            "CarrierStats" => {}
                            //{ "timestamp":"2022-11-29T21:09:30Z", "event":"CarrierJumpRequest", "CarrierID":3704402432, "SystemName":"Ngorowai", "Body":"Ngorowai A", "SystemAddress":4207155286722, "BodyID":1, "DepartureTime":"2022-11-29T21:24:40Z" }
                            "CarrierJumpRequest" => {
                                println!("CarrierJumpRequest: {}",line);
                                let text = format!("__**JUMP INITIATED**__\nDestination: {}\nBody: {}\nDeparture: {}",json["SystemName"],json["Body"],json["DepartureTime"]);
                                self.discord.send_message(self.channel,text.as_str(),"",false).unwrap();
                            }
                            "CarrierTradeOrder" => {}
                            "CarrierFinance" => {}
                            //{ "timestamp":"2022-08-19T17:15:07Z", "event":"CarrierJumpCancelled", "CarrierID":3704402432 }
                            "CarrierJumpCancelled" => {
                                println!("CarrierJumpCancelled: {}",line);
                                let text = format!("__**JUMP CANCELED**__");
                                self.discord.send_message(self.channel,text.as_str(),"",false).unwrap();
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

    let journal_log_file = File::open(&directory_path).expect("file not found!");

    BufReader::new(journal_log_file)
}