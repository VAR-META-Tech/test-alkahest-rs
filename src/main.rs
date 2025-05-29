use std::time::{SystemTime, UNIX_EPOCH};

use alkahest_rs::AlkahestClient;
use alkahest_rs::fixtures::MockERC20Permit;
use alkahest_rs::types::{ArbiterData, Erc20Data};
use alkahest_rs::utils::setup_test_environment;
use alloy::{
    primitives::{Address, Bytes, FixedBytes, U256},
    sol_types::SolValue,
};
#[tokio::main]
async fn main() -> eyre::Result<()> {
    // test setup
    let test = setup_test_environment().await?;

    // give alice some erc20 tokens
    let mock_erc20_a = MockERC20Permit::new(test.mock_addresses.erc20_a, &test.god_provider);
    mock_erc20_a
        .transfer(test.alice.address(), 100.try_into()?)
        .send()
        .await?
        .get_receipt()
        .await?;

    let price = Erc20Data {
        address: test.mock_addresses.erc20_a,
        value: 100.try_into()?,
    };

    // Create custom arbiter data
    let arbiter = test
        .addresses
        .erc20_addresses
        .clone()
        .ok_or(eyre::eyre!("no erc20-related addresses"))?
        .payment_obligation;
    let demand = Bytes::from(b"custom demand data");
    let item = ArbiterData { arbiter, demand };

    let expiration = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 3600; // 1 hour

    // alice deposits tokens to escrow,
    let receipt = test
        .alice_client
        .erc20
        .permit_and_buy_with_erc20(&price, &item, expiration)
        .await?;

    // Verify escrow happened
    let alice_balance = mock_erc20_a
        .balanceOf(test.alice.address())
        .call()
        .await?
        ._0;

    let escrow_balance = mock_erc20_a
        .balanceOf(
            test.addresses
                .erc20_addresses
                .ok_or(eyre::eyre!("no erc20-related addresses"))?
                .escrow_obligation,
        )
        .call()
        .await?
        ._0;

    // all tokens in escrow
    println!("Alice balance: {}", alice_balance);
    println!("Escrow balance: {}", escrow_balance);
    // escrow statement made
    let attested_event = AlkahestClient::get_attested_event(receipt)?;
    assert_ne!(attested_event.uid, FixedBytes::<32>::default());

    Ok(())
}
