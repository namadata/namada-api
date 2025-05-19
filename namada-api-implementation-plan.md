
```# Namada API Implementation Plan

## Overview

This document outlines the implementation plan for building a Rust-based REST API to expose functionality from the Namada SDK, with a focus on the Proof-of-Stake (PoS) module. The API will enable Python (and other language) clients to query the Namada blockchain without needing to implement direct Rust bindings.

A key feature of this API is the ability to connect to any Namada RPC endpoint (local or remote) by configuring the RPC URL at runtime rather than being limited to only local nodes.
```

## Implementation Phases

### Phase 1: Project Setup and Namada RPC Client Configuration

1. **Set up project structure**
   - Initialize Cargo project
   - Configure dependencies
   - Set up logging and error handling

2. **Implement configuration handling**
   - Create configuration struct with RPC URL and other settings
   - Support loading from environment variables (.env file)
   - Support command-line arguments for configuration overrides
   - Add validation for RPC URL format

3. **Implement Namada RPC client**
   - Create a client module that wraps the Namada SDK client
   - Support configurable RPC endpoints (local and remote)
   - Implement connection pooling and reconnection logic
   - Add health check for RPC endpoint connectivity
   - Create an application state that holds the client for use in routes

4. **Implement server initialization**
   - Configure warp router with CORS
   - Set up health check endpoint that verifies RPC connectivity
   - Implement basic error handling middleware
   - Create server startup with graceful shutdown

5. **Create basic endpoints**
   - Implement `/api/health` endpoint (including RPC health)
   - Implement `/api/epoch` endpoint for current epoch


### Phase 2: PoS Module Implementation (Priority)

Focus on the PoS module with priority on the endpoints needed for liveness information and validator lookup by Tendermint address:

1. **Validators Endpoints**

   ```
   GET /api/pos/validators                # Get all validators
   GET /api/pos/validators/{address}      # Get validator details
   GET /api/pos/liveness_info             # Get validators liveness info
   GET /api/pos/validators/tm/{tm_addr}   # Find validator by Tendermint address
   ```

2. **Implementation Details for Key Endpoints**

   a. **Liveness Info Endpoint** (`GET /api/pos/liveness_info`)
   - Maps to the SDK's `liveness_info` function
   - Returns validators consensus participation metrics
   - Include window length and threshold in response

   b. **Validator by Tendermint Address** (`GET /api/pos/validators/tm/{tm_addr}`)
   - Maps to SDK's `validator_by_tm_addr` function
   - Find Namada validator address from Tendermint address
   - Properly handle input validation (important from SDK code review)

   c. **Validator Details** (`GET /api/pos/validators/{address}`)
   - Combine multiple SDK calls:
     - `is_validator`
     - `validator_stake`
     - `validator_commission`
     - `validator_metadata`
     - `validator_state`

3. **Delegations and Bonds Endpoints**

   ```
   GET /api/pos/delegations/{address}             # Get delegations for address
   GET /api/pos/bonds/{source}/to/{validator}     # Get bonds between addresses
   GET /api/pos/rewards/{validator}/{delegator}   # Get rewards earned
   ```

### Phase 3: Extended PoS Functionality

1. **Staking and Validator Set Endpoints**

   ```
   GET /api/pos/validator_sets/consensus          # Get consensus validators
   GET /api/pos/validator_sets/below_capacity     # Get below-capacity validators
   GET /api/pos/total_stake                       # Get total stake in the system
   ```

2. **Enhanced Bond and Unbond Endpoints**

   ```
   GET /api/pos/unbonds/{source}/to/{validator}   # Get unbonding details
   GET /api/pos/withdrawable/{source}/{validator} # Get withdrawable tokens
   ```

### Phase 4: Additional Endpoints and Refinement

1. **Add token-related endpoints**
   - Token balances
   - Token transfers
   - Total supply

2. **Add governance endpoints**
   - Proposals
   - Votes
   - Parameters

3. **API refinements**
   - Add pagination for list endpoints
   - Implement caching for frequently accessed data
   - Add sorting and filtering options



## Deployment Plan

1. **Development Environment**
   - Local development with direct connection to a Namada node

2. **Testing Environment**
   - Dockerize the API for testing
   - Set up CI/CD pipeline for automated testing

3. **Production Deployment**
   - Deploy using Nginx as a reverse proxy
   - Set up SSL with Let's Encrypt
   - Configure systemd service for automatic restart
   - Implement monitoring and logging

4. **DNS Configuration**
   - Create A record pointing to the server IP
   - Set up HTTPS and automatic certificate renewal

## Python Client Example

Once the API is implemented, Python clients can interact with it easily:

```python
import requests

class NamadaClient:
    def __init__(self, base_url="http://localhost:3000/api"):
        self.base_url = base_url
        
    def get_liveness_info(self):
        """Get validator liveness information"""
        response = requests.get(f"{self.base_url}/pos/liveness_info")
        return response.json()
        
    def get_validator_by_tm_addr(self, tm_addr):
        """Find validator by Tendermint address"""
        response = requests.get(f"{self.base_url}/pos/validators/tm/{tm_addr}")
        return response.json()
        
    def get_validator_details(self, address):
        """Get validator details"""
        response = requests.get(f"{self.base_url}/pos/validators/{address}")
        return response.json()
```


## Python Client Example

Once the API is implemented, Python clients can interact with it easily:

```python
import requests

class NamadaClient:
    def __init__(self, base_url="http://localhost:3000/api"):
        self.base_url = base_url
        
    def check_health(self):
        """Check if the API is running"""
        response = requests.get(f"{self.base_url}/health")
        return response.json()
        
    def check_rpc_health(self):
        """Check if the Namada RPC is connected"""
        response = requests.get(f"{self.base_url}/health/rpc")
        return response.json()
        
    def get_epoch(self):
        """Get current epoch"""
        response = requests.get(f"{self.base_url}/epoch")
        return response.json()
        
    def get_liveness_info(self):
        """Get validator liveness information"""
        response = requests.get(f"{self.base_url}/pos/liveness_info")
        return response.json()
        
    def get_validator_by_tm_addr(self, tm_addr):
        """Find validator by Tendermint address"""
        response = requests.get(f"{self.base_url}/pos/validators/tm/{tm_addr}")
        return response.json()
        
    def get_validator_details(self, address):
        """Get validator details"""
        response = requests.get(f"{self.base_url}/pos/validators/{address}")
        return response.json()
        
    def get_validators(self):
        """Get all validators"""
        response = requests.get(f"{self.base_url}/pos/validators")
        return response.json()
```

Example usage:

```python
# Connect to a specific Namada API endpoint
client = NamadaClient("https://api.namada-example.com/api")

# Check if API and RPC are healthy
api_health = client.check_health()
rpc_health = client.check_rpc_health()
print(f"API status: {api_health['status']}")
print(f"RPC status: {rpc_health['status']}, URL## Main Server Setup

