use goose::prelude::*;

async fn list_houses(user: &mut GooseUser) -> TransactionResult {
    let _goose = user.get("/api/v1/houses").await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .register_scenario(scenario!("BrowseHouses").register_transaction(transaction!(list_houses).set_name("list_houses")))
        .set_default(GooseDefault::Host, "http://localhost:8080")?
        .execute()
        .await?;
    Ok(())
}
