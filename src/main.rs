use std::thread::sleep;
use std::time::Duration;
use discord::Discord;
use discord::model::ChannelId;
use dotenv::dotenv;

mod journal_reader;

fn main() {
    dotenv().expect(".env file not found");
    let discord = Discord::from_bot_token(std::env::var("BOT_TOKEN").unwrap().as_str()).expect("Login failed");
    let channel = ChannelId(std::env::var("CHANNEL_ID").unwrap().parse().unwrap());
    let mut journal_reader = journal_reader::initialize(std::env::var("JOURNAL_PATH").unwrap(),discord,channel);
    let timeout: u64 = std::env::var("TIMEOUT").unwrap().parse().unwrap();
    println!("Ready");
    loop {
        journal_reader.run();
        sleep(Duration::from_millis(timeout));
    }
}
