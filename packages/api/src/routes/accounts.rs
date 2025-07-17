use poem::{handler, web::Json, Result};
use store::{models::Account, Store};

#[handler]
pub async fn get_accounts() -> Result<Json<Vec<Account>>> {
    let mut store = Store::default();

    match store.get_accounts() {
        Ok(accounts) => Ok(Json(accounts)),
        Err(e) => {
            eprintln!("Error fetching accounts: {}", e);
            Ok(Json(vec![]))
        }
    }
}
