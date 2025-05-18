# Namada REST API

A professional, production-ready REST API scaffold for the Namada blockchain, built with Rust. This project provides a robust foundation for exposing Namada SDK functionality through REST endpoints, currently implementing Proof-of-Stake module (PoS) features with the ability to extend to all Namada SDK capabilities.

The purpose of this project is to give non-Rust developers an easy way to interact with the Namada SDK queries - facilitating greater off chain integrations and tools for the Namada ecosystem.

## Overview

This API serves as a bridge between the Namada blockchain and web applications, offering a clean, REST interface to Namada's SDK queries. 

## Current Features
- Complete PoS (Proof-of-Stake) API implementation
  - Validator liveness monitoring
  - Validator lookup and details
  - Delegation information
  - Comprehensive validator listing
- Health monitoring endpoints
- Production-ready configuration management
- Error handling
- Type-safe responses

## Architecture

The project is structured to make it easy to add new endpoints and functionality:

```
src/
├── api/          # API route handlers
├── config/       # Configuration management
├── models/       # Data models and types
├── services/     # Business logic
└── utils/        # Shared utilities
```

## Configuration

The API supports multiple configuration methods, following the principle of least surprise:

1. Environment variables
2. `.env` file
3. Command-line arguments

Key configuration options:
- `NAMADA_RPC_URL`: Namada RPC endpoint (default: `http://localhost:26657`)
- `API_PORT`: API server port (default: `3000`)

Example configuration:
```sh
# .env file
NAMADA_RPC_URL=http://localhost:26657
API_PORT=3000
```

## Running the API

### Development
```sh
cargo run
```

### Production
```sh
cargo run --release
```

### Docker (Coming Soon)
```sh
docker run namada-api
```

## API Endpoints

### Docs
- `GET /api/docs` — API documentation

### Health
- `GET /api/health` — API health check
- `GET /api/health/rpc` — Namada RPC health check

### PoS (Current Implementation)
- `GET /api/pos/liveness_info` — Validator liveness information
- `GET /api/pos/validators/tm/{tm_addr}` — Validator lookup by Tendermint address
- `GET /api/pos/validators/{address}` — Detailed validator information
- `GET /api/pos/validators` — List all validators
- `GET /api/pos/delegations/{address}` — Get delegations for an address

## Client Libraries

### Python
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
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## Roadmap

- [ ] Additional Namada SDK endpoints
- [ ] systemd service file / docker container
- [ ] tests for futureproofing
- [ ] Rate limiting and caching
- [ ] Metrics and monitoring

## Dependencies

- Namada SDK: 0.149.1
- Rust: Latest stable
- Additional dependencies listed in `Cargo.toml`

## License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.


## Support

For support, please open an issue in the GitHub repository.