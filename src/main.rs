extern crate pretty_env_logger;

pub mod models;

use weid_light_client::WeIdGenerator;

use serde_json::{Value};
use weid_light_client::GenerateWeIdError;

use std::env;

#[macro_use] extern crate log;

fn main(){
    pretty_env_logger::init();
    let url = env::var("WEID_URL").expect("DATABASE_URL must be set");
    let weid_generator = WeIdGenerator::new(url.to_string());
    gen_weid_online_and_save(weid_generator);
}

fn gen_weid_online_and_save(weid_generator: WeIdGenerator) -> Result<Value, GenerateWeIdError>{
    let result = weid_generator.create_weid_online();

    match result {
        Ok(payload) => {
            let weid_str: String = 
                payload["respBody"]
                .to_string()
                .replace("\"", "");
            let vec: Vec<&str> = 
                weid_str
                .split(":")
                .collect();

            let chain_id: i32 = vec[2].parse().unwrap();
            let addr: &str = vec[3];
            // create data
            let sqlite_conn = models::establish_connection();
            
            models::save_weid(&sqlite_conn, chain_id, addr);
            info!("gen and save weid to local {}.", weid_str);
            Ok(payload)
        },
        Err(e) => {
            info!("{}", e);
            Err(e)
        }
    }
}



