use chrono::NaiveDateTime;
#[cfg(test)]
use diesel::debug_query;
use diesel::insert_into;
use diesel::prelude::*;
#[cfg(test)]
use diesel::sqlite::Sqlite;
use serde_derive::Deserialize;
use std::error::Error;

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
pub struct NewWeid {
    pub chain_id: i32,
    pub addr: String,
}

#[derive(Queryable, PartialEq, Debug)]
pub struct Weid {
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
