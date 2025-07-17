use poem::{handler, web::Json, Result};
use store::{models::Slot, Store};

#[handler]
pub async fn get_slots() -> Result<Json<Vec<Slot>>> {
    let mut store = Store::default();

    match store.get_slots() {
        Ok(slots) => Ok(Json(slots)),
        Err(e) => {
            eprintln!("Error fetching slots: {}", e);
            Ok(Json(vec![]))
        }
    }
}
