use diesel::{Connection, PgConnection};
use std::sync::{Arc, Mutex};


pub mod schema;

#[derive(Clone)]
pub struct ServerState {
    pub pgconnection: Arc<Mutex<PgConnection>>,
}

pub mod db_type {
    use serde::{Deserialize, Serialize};
    use diesel::prelude::Queryable;

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

    #[derive(Insertable, Deserialize, Serialize, Clone)]
    #[diesel(table_name = account)]
    pub struct NewAccount<'a> {
        username: &'a str,
        password: &'a str,
    }
}

pub fn establish_server_state() -> anyhow::Result<ServerState> {
    // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pgconnection = diesel::PgConnection::establish(include_str!("..\\..\\.env"))?;

    Ok(ServerState {pgconnection: Arc::new(Mutex::new(pgconnection))})
}