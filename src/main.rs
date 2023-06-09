use std::thread::sleep;
use std::time::Duration;
use discord::Discord;
use discord::model::ChannelId;
use dotenv::dotenv;

mod journal_reader;

fn main() {
    dotenv().expect(".env file not found");
    let discord = Discord::from_bot_token(std::env::var("BOT_TOKEN").unwrap().as_str()).expect("Login failed");
    let (mut connection, _) = discord.connect().expect("Connect failed");
    connection.set_game_name("Cruising in the void".to_string());
    let channel = ChannelId(std::env::var("CHANNEL_ID").unwrap().parse().unwrap());
    let mut journal_reader = journal_reader::initialize(std::env::var("JOURNAL_PATH").unwrap(),discord,connection,channel);
    println!("Ready");
    loop {
        sleep(Duration::from_secs(1));
        journal_reader.run();
        //let _ = connection.recv_event();
    }
}
