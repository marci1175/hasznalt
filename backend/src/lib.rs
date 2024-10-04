use diesel::{Connection, PgConnection};
use std::sync::{Arc, Mutex};

pub mod schema;

#[derive(Clone)]
pub struct ServerState {
    pub pgconnection: Arc<Mutex<PgConnection>>,
}

pub mod db_type {
    use diesel::prelude::{Insertable, Queryable, QueryableByName};
    use serde::{Deserialize, Serialize};

    use crate::schema::account;

    #[derive(QueryableByName, Queryable, Insertable, Deserialize, Serialize, Clone)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    #[diesel(table_name = account)]
    pub struct Account {
        pub id: i32,
        pub username: String,
        pub password: String,
        pub created_at: chrono::NaiveDateTime,
    }
}

pub fn establish_server_state() -> anyhow::Result<ServerState> {
    let pgconnection = diesel::PgConnection::establish(include_str!("..\\..\\.env"))?;

    Ok(ServerState {
        pgconnection: Arc::new(Mutex::new(pgconnection)),
    })
}
