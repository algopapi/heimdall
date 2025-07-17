use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::slots)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Slot {
    pub id: i32,
    pub slot: i64,
    pub parent: Option<i64>,
    pub status: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::slots)]
pub struct NewSlot {
    pub slot: i64,
    pub parent: Option<i64>,
    pub status: i32,
}

impl Slot {
    pub fn new(
        slot: i64,
        parent: Option<i64>,
        status: i32,
    ) -> NewSlot {
        NewSlot {
            slot,
            parent,
            status,
        }
    }
}
