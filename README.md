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
docs/
├── api.html      # API documentation
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

### Install dependencies - Linux 

```sh
sudo apt update
sudo apt install protobuf-compiler libssl-dev pkg-config build-essential
```

### Development
```sh
cargo run
```

### Production Build
```sh
# Build optimized release binary
cargo build --release

# The binary will be available at target/release/namada-api
./target/release/namada-api
```

## Production Deployment

### Systemd Service Deployment

For production deployments on Linux systems using systemd:

1. **Build the release binary:**
```sh
cargo build --release
```

2. **Install and configure the systemd service:**
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
sudo journalctl -u namada-api.service -f
```

The service file is configured to:
- Run as root user (modify as needed for your security requirements)
- Install the application in `/opt/namada-api`
- Set environment variables as defined in the `.env` file
- Automatically restart on failure
- Start after network is available

**Note:** You can modify the `namada-api.service` file to change the user, working directory, or environment variables as needed for your deployment.

### Docker (Coming Soon)
```sh
docker run namada-api
```

## API Endpoints

### Documentation
- `GET /api/docs` — Interactive API documentation

### Health
- `GET /api/health/api_status` — API health check
- `GET /api/health/rpc_status` — Namada RPC health check

### Proof of Stake
- `GET /api/pos/liveness_info` — Validator liveness information
- `GET /api/pos/validator_by_tm_addr/{tm_addr}` — Validator lookup by Tendermint address
- `GET /api/pos/validator_details/{address}` — Detailed validator information
- `GET /api/pos/validators` — List all validators (addresses only)
- `GET /api/pos/validators_details` — Detailed information for all validators (paginated)
- `GET /api/pos/validator_set/consensus` — Consensus validator set
- `GET /api/pos/validator_set/below_capacity` — Below-capacity validator set

## Client Libraries

### Python
```python
import requests

class NamadaClient:
    def __init__(self, base_url="http://localhost:3000/api"):
        self.base_url = base_url
    
    def check_api_health(self):
        """Check if the API service is running"""
        return requests.get(f"{self.base_url}/health/api_status").json()
    
    def check_rpc_health(self):
        """Check if the API can connect to Namada RPC"""
        return requests.get(f"{self.base_url}/health/rpc_status").json()
    
    def get_liveness_info(self):
        """Get validator liveness information"""
        return requests.get(f"{self.base_url}/pos/liveness_info").json()
    
    def get_validator_by_tm_addr(self, tm_addr):
        """Get validator by Tendermint address"""
        return requests.get(f"{self.base_url}/pos/validator_by_tm_addr/{tm_addr}").json()
    
    def get_validator_details(self, address):
        """Get detailed information about a specific validator"""
        return requests.get(f"{self.base_url}/pos/validator_details/{address}").json()
    
    def get_validators(self):
        """Get list of all validators (addresses only)"""
        return requests.get(f"{self.base_url}/pos/validators").json()
    
    def get_validators_details(self, page=1, per_page=10):
        """Get detailed information for all validators with pagination"""
        params = {"page": page, "per_page": per_page}
        return requests.get(f"{self.base_url}/pos/validators_details", params=params).json()
    
    def get_consensus_validator_set(self):
        """Get consensus validator set"""
        return requests.get(f"{self.base_url}/pos/validator_set/consensus").json()
    
    def get_below_capacity_validator_set(self):
        """Get below-capacity validator set"""
        return requests.get(f"{self.base_url}/pos/validator_set/below_capacity").json()

# Example usage
client = NamadaClient()

# Check health
print(client.check_api_health())
print(client.check_rpc_health())

# Get validator information
validators = client.get_validators()
print(f"Found {len(validators['validators'])} validators")

# Get detailed validator information with pagination
details = client.get_validators_details(page=1, per_page=5)
print(f"Page 1 of {details['pagination']['total_pages']}")
```

### JavaScript/Node.js
```javascript
class NamadaClient {
    constructor(baseUrl = "http://localhost:3000/api") {
        this.baseUrl = baseUrl;
    }

    async checkApiHealth() {
        const response = await fetch(`${this.baseUrl}/health/api_status`);
        return response.json();
    }

    async getValidatorDetails(address) {
        const response = await fetch(`${this.baseUrl}/pos/validator_details/${address}`);
        return response.json();
    }

    async getValidatorsDetails(page = 1, perPage = 10) {
        const response = await fetch(`${this.baseUrl}/pos/validators_details?page=${page}&per_page=${perPage}`);
        return response.json();
    }
}
```

## Development

### Project Structure
```
namada-api/
├── src/
│   ├── models/         # Data models and response types
│   ├── client.rs       # Namada SDK client wrapper
│   ├── config.rs       # Configuration management
│   └── main.rs         # Main application and routes
├── docs/
│   └── api.html        # API documentation
├── tests/              # Integration tests
├── Cargo.toml          # Rust dependencies
└── README.md           # This file
```

### Running Tests
```sh
cargo test
```


## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Guidelines
- Follow Rust best practices and idioms
- Add tests for new functionality
- Update documentation for API changes
- Use meaningful commit messages

## Roadmap

- [ ] Additional Namada SDK endpoints (governance, IBC, etc.)
- [ ] Docker container and Kubernetes manifests
- [ ] Comprehensive test suite
- [ ] Rate limiting and caching
- [ ] Metrics and monitoring (Prometheus/Grafana)
- [ ] WebSocket support for real-time updates
- [ ] API versioning strategy

## Dependencies

- **Namada SDK**: 0.149.1
- **Rust**: Latest stable (1.70+)
- **Additional dependencies**: Listed in `Cargo.toml`

## Performance

The API is designed for production use with:
- Async/await for non-blocking operations
- Connection pooling for RPC calls
- Efficient JSON serialization
- Minimal memory footprint

## Security

- Input validation on all endpoints
- Rate limiting (planned)
- CORS support
- No sensitive data logging

## License

This project is licensed under the GPL-3.0 License - see the [LICENSE](LICENSE) file for details.

## Support

For support, please:
1. Check the [API documentation](http://localhost:3000/api/docs) when running locally
2. Review existing [GitHub issues](https://github.com/your-repo/namada-api/issues)
3. Open a new issue with detailed information about your problem

## Acknowledgments

- Built with the [Namada SDK](https://github.com/anoma/namada)
- Powered by [Warp](https://github.com/seanmonstar/warp) web framework
- Inspired by the Namada community's need for accessible blockchain data
