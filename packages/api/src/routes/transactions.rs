use poem::{handler, web::Json, Result};
use store::{models::Transaction, Store};

#[handler]
pub async fn get_transactions() -> Result<Json<Vec<Transaction>>> {
    let mut store = Store::default();

    match store.get_transactions() {
        Ok(transactions) => Ok(Json(transactions)),
        Err(e) => {
            eprintln!("Error fetching transactions: {}", e);
            Ok(Json(vec![]))
        }
    }
}
