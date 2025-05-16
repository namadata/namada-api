# Namada PoS REST API

This project provides a REST API for querying Namada Proof-of-Stake (PoS) validator, liveness, and delegation information, using the Namada SDK and modules at version 0.149.1.

## Features
- Query validator liveness info
- Lookup validator by Tendermint address
- Get validator details
- List all validators
- Get delegations for an address
- Health checks for API and Namada RPC

## Configuration

You can configure the API via environment variables, CLI arguments, or a config file (see `src/config.rs`).

- `NAMADA_RPC_URL` (env/CLI): The Namada RPC endpoint (default: `http://localhost:26657`)
- `API_PORT` (env/CLI): The port to run the API server on (default: `3000`)

Example:
```sh
NAMADA_RPC_URL=http://localhost:26657 cargo run --release -- --port 3000
```

## Running the API

You can run the API in several ways:

### 1. Using a `.env` file (recommended for local dev)
Create a `.env` file in your project root:
```
NAMADA_RPC_URL=http://localhost:26657
API_PORT=3000
```
Then start the server:
```sh
cargo run --release
```

### 2. Using environment variables
```sh
export NAMADA_RPC_URL=http://localhost:26657
export API_PORT=3000
cargo run --release
```

### 3. Using CLI arguments
```sh
cargo run --release -- --rpc-url http://localhost:26657 --port 3000
```

**Configuration precedence:**
1. CLI arguments (highest)
2. Environment variables (including from `.env`)
3. Defaults in code

## Endpoints

All endpoints are under `/api`.

### Health
- `GET /api/health` — API health
- `GET /api/health/rpc` — Namada RPC health

### PoS
- `GET /api/pos/liveness_info` — Validator liveness info
- `GET /api/pos/validators/tm/{tm_addr}` — Lookup validator by Tendermint address
- `GET /api/pos/validators/{address}` — Validator details
- `GET /api/pos/validators` — List all validators
- `GET /api/pos/delegations/{address}` — Delegations for an address

## Example Python Client

```python
import requests

class NamadaClient:
    def __init__(self, base_url="http://localhost:3000/api"):
        self.base_url = base_url
    def get_liveness_info(self):
        return requests.get(f"{self.base_url}/pos/liveness_info").json()
    def get_validator_by_tm_addr(self, tm_addr):
        return requests.get(f"{self.base_url}/pos/validators/tm/{tm_addr}").json()
    def get_validator_details(self, address):
        return requests.get(f"{self.base_url}/pos/validators/{address}").json()
    def get_validators(self):
        return requests.get(f"{self.base_url}/pos/validators").json()
    def get_delegations(self, address):
        return requests.get(f"{self.base_url}/pos/delegations/{address}").json()

# Usage
client = NamadaClient()
print(client.get_liveness_info())
```

## Namada SDK Version
- SDK and all modules: **0.149.1**

## License
GPL-3.0 