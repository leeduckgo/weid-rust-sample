use chrono::NaiveDateTime;
#[cfg(test)]
use diesel::debug_query;
use diesel::insert_into;
use diesel::prelude::*;

use std::env;
use dotenv::dotenv;

#[cfg(test)]
use diesel::sqlite::Sqlite;

pub mod schema {
    diesel::table! {
        weids {
            id -> Integer,
            chain_id -> Integer,
            addr -> Text,

            created_at -> Timestamp,
            updated_at -> Timestamp,
        }
    }
}

use schema::weids;

#[derive(Insertable)]
#[table_name = "weids"]
pub struct NewWeId {
    pub chain_id: i32,
    pub addr: String,
}

#[derive(Queryable, PartialEq, Debug)]
pub struct WeId {
    pub id: i32,
    pub chain_id: i32,
    pub addr: String,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime, 
}

pub fn insert_default_values(conn: &SqliteConnection) -> QueryResult<usize> {
    use schema::weids::dsl::*;

    insert_into(weids).default_values().execute(conn)
}

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn save_weid(conn: &SqliteConnection, c_id: i32, address: &str) -> usize {
    
    let new_weid = NewWeId {chain_id: c_id, addr: address.to_string()};

    diesel::insert_into(weids::table)
        .values(&new_weid)
        .execute(conn)
        .expect("Error saving new weid")
}
