use diesel::{Connection, PgConnection};

use crate::config::Config;

pub struct Store {
    pub conn: PgConnection,
}

impl Default for Store {
    fn default() -> Self {
        let database_url = Config::default().db_url;

        let connection =
            PgConnection::establish(&database_url).expect("Error connecting to the database");

        Self { conn: connection }
    }
}

impl Store {
    pub fn create_user(&self) {}
}
