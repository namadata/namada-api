# Namada API

A REST API for interacting with the Namada blockchain, with a focus on the Proof-of-Stake (PoS) module.

## Features

- Connect to any Namada RPC endpoint (local or remote)
- Query validator information
- Get liveness information
- Look up validators by Tendermint address
- Query current epoch and native token

## Getting Started

### Prerequisites

- Rust and Cargo
- Access to a Namada RPC endpoint

### Installation

1. Clone the repository
2. Build the project:

```bash
cargo build --release
```

### Configuration

The API can be configured using:

1. Environment variables
2. Command-line arguments
3. Configuration file

#### Environment Variables

- `NAMADA_RPC_URL`: URL of the Namada RPC endpoint
- `API_PORT`: Port for the API server (default: 3000)

#### Command-Line Arguments

```bash
namada-api --rpc-url http://localhost:26657 --port 3000
```

### Running the API

There are several ways to run the Namada API:

#### 1. Using Cargo

The simplest way to run the API during development:

```bash
cargo run --release -- --rpc-url http://localhost:26657
```

#### 2. Running the compiled binary

After building the release version:

```bash
./target/release/namada-api --rpc-url http://localhost:26657
```

#### 3. Using environment variables

You can set environment variables instead of command-line arguments:

```bash
export NAMADA_RPC_URL=http://localhost:26657
export API_PORT=3000
cargo run --release
```

#### 4. Using a .env file

Create a `.env` file in the project root:

```
NAMADA_RPC_URL=http://localhost:26657
API_PORT=3000
```

Then run:

```bash
cargo run --release
```

## API Endpoints

### Health Checks

- `GET /api/health`: Check API health
- `GET /api/health/rpc`: Check RPC connection health

### Basic Information

- `GET /api/epoch`: Get current epoch
- `GET /api/native_token`: Get native token address

### PoS Endpoints

- `GET /api/pos/validators`: Get all validators
- `GET /api/pos/validators/{address}`: Get validator details
- `GET /api/pos/liveness_info`: Get validators liveness info
- `GET /api/pos/validators/tm/{tm_addr}`: Find validator by Tendermint address

## Example Usage

Once the API is running, you can test it using curl:

```bash
# Check API health
curl http://localhost:3000/api/health

# Check RPC connection
curl http://localhost:3000/api/health/rpc

# Get current epoch
curl http://localhost:3000/api/epoch

# Get validators liveness info
curl http://localhost:3000/api/pos/liveness_info
```

## License

[MIT](LICENSE) 