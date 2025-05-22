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
├── models/       # Data models and types
├── tests/        # Tests
├── client.rs     # SDK Client
├── config.rs     # Configuration management
├── main.rs       # Main and routes
├── state.rs      # App state / Namada Client handling 
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

### Install dependencies - linux 

```sh
sudo apt update
sudo apt install protobuf-compiler libssl-dev pkg-config build-essential
```

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

## Systemd Service Deployment

For deployments on Linux systems using systemd, a service file is included:

```sh
# Copy the service file to systemd directory
sudo cp namada-api.service /etc/systemd/system/

# Reload systemd to recognize the new service
sudo systemctl daemon-reload

# Enable the service to start automatically on boot
sudo systemctl enable namada-api.service

# Start the service
sudo systemctl start namada-api.service

# Check the status
sudo systemctl status namada-api.service

# View logs
sudo journalctl -u namada-api.service
```

The service file is configured to:
- Run as root user
- Install the application in `/root/namada-api`
- Set the same environment variables as defined in the `.env` file
- Automatically restart on failure

You can modify the `namada-api.service` file to change these settings if needed.

## API Endpoints

### Docs
- `GET /api/docs` — API documentation

### Health
- `GET /api/health/api_status` — API health check
- `GET /api/health/rpc_status` — Namada RPC health check

### PoS (Current Implementation)
- `GET /api/pos/liveness_info` — Validator liveness information
- `GET /api/pos/validator_by_tm_addr/{tm_addr}` — Validator lookup by Tendermint address
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
    
    def check_api_health(self):
        return requests.get(f"{self.base_url}/health/api_status").json()
    
    def check_rpc_health(self):
        return requests.get(f"{self.base_url}/health/rpc_status").json()
    
    def get_liveness_info(self):
        return requests.get(f"{self.base_url}/pos/liveness_info").json()
    
    def get_validator_by_tm_addr(self, tm_addr):
        return requests.get(f"{self.base_url}/pos/validator_by_tm_addr/{tm_addr}").json()
    
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
