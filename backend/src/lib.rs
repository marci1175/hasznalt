use diesel::{Connection, PgConnection};
use std::sync::{Arc, Mutex};

pub mod schema;

#[derive(Clone)]
pub struct ServerState {
    pub pgconnection: Arc<Mutex<PgConnection>>,
}

pub mod db_type {
    use diesel::prelude::Queryable;
    use serde::{Deserialize, Serialize};

    #[derive(Queryable, Deserialize, Serialize, Clone)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct Account {
        pub id: i32,
        pub username: String,
        pub password: String,
        pub created_at: chrono::NaiveDateTime,
    }
}

pub mod rs_type {
    use diesel::Insertable;
    use serde::{Deserialize, Serialize};

    use crate::schema::account;

    #[derive(Insertable, Deserialize, Serialize, Clone, Debug)]
    #[diesel(table_name = account)]
    pub struct NewAccount {
        pub username: String,
        pub password: String,
    }
}

pub fn establish_server_state() -> anyhow::Result<ServerState> {
    let pgconnection = diesel::PgConnection::establish(include_str!("..\\..\\.env"))?;

    Ok(ServerState {
        pgconnection: Arc::new(Mutex::new(pgconnection)),
    })
}
