use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::protocols)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Protocol {
    pub id: i32,
    pub name: String,
    pub program_id: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::protocols)]
pub struct NewProtocol {
    pub name: String,
    pub program_id: String,
    pub description: Option<String>,
    pub is_active: bool,
}

impl Protocol {
    pub fn new(
        name: String,
        program_id: String,
        description: Option<String>,
        is_active: bool,
    ) -> NewProtocol {
        NewProtocol {
            name,
            program_id,
            description,
            is_active,
        }
    }
}
