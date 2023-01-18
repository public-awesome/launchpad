# E2E tests

Gas profiled integration tests hitting a real, local Stargaze chain.

## Running Tests 

Use `make e2etest-full` to run the tests or `ENABLE_MAX_COLLECTION=true make e2etest-full` to run all the tests, including the exhaustive minting tests.

### Dev Loop

It takes a long time to compile / optimize the contracts, so when developing the integration tests, its best to just `make e2etest`.

Or if you only want to run one test: `make e2etest TEST_NAME=test_invalid_start_trading_time`.

## Adding New Integration Tests

Add new tests in `src/tests`:
```rust
#[test_context(Chain)]
#[test]
#[ignore]
fn test_mint(chain: &mut Chain) {
    let user = chain.cfg.users[0].clone();

    instantiate_factory(chain, &user.account.address.to_string(), &user.key).unwrap();

    let minter_msg = Sg2ExecuteMsg::CreateMinter(create_minter_msg(
            chain,
            &user.account.address.to_string(),
            MAX_TOKENS,
            50,
            start_time,
            None,
        ));

    let res = chain
        .orc
        .execute(
            FACTORY_NAME,
            "factory_exec_minter_inst",
            &minter_msg,
            &user.key,
            vec![],
        )
        .unwrap();

    ...
}
```

We are currentlying
[ignoring](https://doc.rust-lang.org/book/ch11-02-running-tests.html#ignoring-some-tests-unless-specifically-requested)
all integration tests by adding the `#[ignore]` annotation to them,
because we want to skip them when people run `cargo test` from the
workspace root.