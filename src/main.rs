extern crate pretty_env_logger;

pub mod models;

use diesel::prelude::*;
use std::env;
use dotenv::dotenv;

use weid_light_client::WeIdRestService;
use models::*;

use models::schema::weids;
use models::schema::weids::dsl::*;

#[macro_use] extern crate log;

fn main(){
    pretty_env_logger::init(); 

    // create data
    let sqlite_conn = establish_connection();
    create_weid(&sqlite_conn, 1, "34be11396f3a91c5Ab5A1220e756C6300FB2b20a");
    
    // query data
    let results = weids.load::<Weid>(&sqlite_conn)
        .expect("Error loading weids");
    // log weids
    info!("Displaying {} weids", results.len());
    for weid in results{
        info!("did:weid:{}:{}", weid.chain_id, weid.addr);
    }
}

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_weid(conn: &SqliteConnection, c_id: i32, address: &str) -> usize {
    

    let new_weid = NewWeid {chain_id: c_id, addr: address.to_string()};

    diesel::insert_into(weids::table)
        .values(&new_weid)
        .execute(conn)
        .expect("Error saving new weid")
}