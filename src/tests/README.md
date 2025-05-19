# Namada API Tests

This directory contains tests for the Namada API endpoints.

## Structure

- `mod.rs` - Main test module file that exports submodules
- `health_tests.rs` - Tests for health endpoints
- `pos_tests.rs` - Tests for Proof of Stake (PoS) endpoints

## Running Tests

To run all tests:

```bash
cargo test
```

To run tests for a specific module:

```bash
cargo test --test health_tests
cargo test --test pos_tests
```

To run a specific test:

```bash
cargo test test_get_validator_details
```

## Test Design Philosophy

The tests in this project follow these principles:

1. **Isolation**: Each test focuses on a single endpoint and its behavior.
2. **No external dependencies**: Tests use a mock Namada client that doesn't connect to a real node.
3. **Flexible assertions**: Tests expect either correct behavior or appropriate error responses.
4. **Complete API coverage**: Every API endpoint has at least one test.

## Test Implementation

Each test follows a common pattern:

1. Create a test client with a mock URL (via `setup_test_client()`)
2. Set up the warp filter for the endpoint being tested
3. Make a test request to the endpoint with appropriate parameters
4. Assert on the response structure and status codes

Since we're using a mock URL that won't connect to a real Namada node, most tests expect a 500 status code for valid inputs (since the RPC call will fail). For invalid inputs, we expect either a 400 (Bad Request) or a 500 (Internal Server Error) depending on implementation details.

## Test Categories

### Health Tests (`health_tests.rs`)

- `test_health_check`: Tests the basic health endpoint (`/api/health`)
- `test_rpc_health_check`: Tests the RPC health check endpoint (`/api/health/rpc`)

### Proof of Stake Tests (`pos_tests.rs`)

- `test_get_all_validators`: Tests retrieving all validators
- `test_get_validator_details`: Tests getting details for a specific validator
- `test_get_validators_details_pagination_validation`: Tests pagination for validator details
- `test_get_validator_by_tm_addr`: Tests finding a validator by Tendermint address
- `test_get_liveness_info`: Tests retrieving validator liveness information
- `test_get_consensus_validator_set`: Tests getting the consensus validator set
- `test_get_below_capacity_validator_set`: Tests getting the below-capacity validator set

## Mock Test Client

The `setup_test_client()` function creates a mock test client with a non-existent URL. This approach ensures that tests don't depend on external services, but it means that our tests primarily verify:

1. The routes are correctly configured
2. Validation logic works as expected
3. Error handling behaves appropriately

For more comprehensive testing, a proper mocked client that returns predefined responses could be implemented.

## Future Test Improvements

Potential improvements to the test suite:

1. Create a mock RPC client that returns predetermined responses
2. Add integration tests that connect to a test Namada network
3. Add property-based tests for more robust validation testing
4. Implement test coverage reporting 