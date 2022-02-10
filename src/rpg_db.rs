use crate::models::{NewSourceCode, SourceCode, Users};
use diesel::prelude::*;
use std::env;

/// Returns db connection
pub fn establish_connection() -> SqliteConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

/// Creating new source
pub fn create_source(conn: &mut SqliteConnection, source_code: &str, author: Users) -> SourceCode {
    NewSourceCode::new(source_code.to_string(), author).save(conn)
}
