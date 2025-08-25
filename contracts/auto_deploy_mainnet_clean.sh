#!/bin/bash

# Auto Deploy to Mainnet Script - Clean Version for OpenZeppelin + STRK
# This script automatically sets up everything needed for mainnet deployment

set -e

echo "🚀 Auto Deploy to Starknet Mainnet"
echo "=================================="
echo "This script will set up everything needed for mainnet deployment"
echo "Using OpenZeppelin accounts and STRK for fees"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Check if sncast is available
if ! command -v sncast &> /dev/null; then
    print_error "sncast is not installed or not in PATH"
    echo "Install Starknet Foundry from: https://foundry-rs.github.io/starknet-foundry/getting-started/installation.html"
    exit 1
fi

# Check if scarb is available
if ! command -v scarb &> /dev/null; then
    print_error "scarb is not installed or not in PATH"
    echo "Install it from: https://docs.swmansion.com/scarb/docs/installing-scarb"
    exit 1
fi

print_status "Tools check passed"

# Set up environment for mainnet
export STARKNET_RPC=https://starknet-mainnet.public.blastapi.io/rpc/v0_7
export STARKNET_NETWORK=mainnet

print_info "Using RPC: $STARKNET_RPC"
print_info "Network: Mainnet"

# Build the contract
echo ""
print_info "Building contract..."
if ! scarb build; then
    print_error "Contract build failed"
    exit 1
fi

print_status "Contract built successfully"

# Check if account already exists
ACCOUNT_FILE="$HOME/.starkli/accounts.json"
if [ -f "$ACCOUNT_FILE" ]; then
    echo ""
    print_warning "Account file already exists: $ACCOUNT_FILE"
    read -p "Do you want to use the existing account? (yes/no): " use_existing
    if [ "$use_existing" = "yes" ]; then
        print_status "Using existing account"
    else
        print_info "Will create new account"
        rm -f "$ACCOUNT_FILE"
    fi
fi

# Account setup
if [ ! -f "$ACCOUNT_FILE" ]; then
    echo ""
    print_info "Creating new OpenZeppelin account for mainnet deployment"
    
    # Create account
    echo ""
    print_info "Creating OpenZeppelin account..."
    ACCOUNT_OUTPUT=$(sncast account create --name oz_account --type oz 2>&1)
    
    if [ $? -ne 0 ]; then
        print_error "Account creation failed"
        echo "$ACCOUNT_OUTPUT"
        exit 1
    fi
    
    # Extract account address from output
    ACCOUNT_ADDRESS=$(echo "$ACCOUNT_OUTPUT" | grep "address:" | awk '{print $2}')
    MAX_FEE=$(echo "$ACCOUNT_OUTPUT" | grep "max_fee:" | awk '{print $2}')
    
    print_status "Account created successfully!"
    echo "Account Address: $ACCOUNT_ADDRESS"
    echo "Required funding: $MAX_FEE wei STRK (approximately $(echo "scale=2; $MAX_FEE / 1000000000000000000" | bc -l) STRK)"
    echo ""
    
    print_warning "IMPORTANT: You need to fund this account before deployment!"
    echo "1. Send at least $(echo "scale=0; $MAX_FEE / 1000000000000000000 + 100" | bc -l) STRK tokens to: $ACCOUNT_ADDRESS"
    echo "2. You can buy STRK on exchanges like:"
    echo "   - Binance, Coinbase, OKX, etc."
    echo "   - Or bridge from Ethereum using StarkGate: https://starkgate.starknet.io/"
    echo "3. Wait for the transaction to confirm (usually 1-2 minutes)"
    echo ""
    echo "Useful links:"
    echo "- Check balance: https://starkscan.co/contract/$ACCOUNT_ADDRESS"
    echo "- StarkGate Bridge: https://starkgate.starknet.io/"
    echo "- STRK Token on Ethereum: 0xCa14007Eff0dB1f8135f4C25B34De49AB0d42766"
    echo ""
    
    read -p "Press Enter after you have funded the account and want to deploy it..."
    
    # Deploy account
    print_info "Deploying account to mainnet..."
    DEPLOY_ACCOUNT_OUTPUT=$(sncast account deploy --name oz_account --fee-token strk 2>&1)
    
    if [ $? -ne 0 ]; then
        print_error "Account deployment failed"
        echo "$DEPLOY_ACCOUNT_OUTPUT"
        echo ""
        print_info "Please ensure the account has sufficient STRK tokens and try again."
        exit 1
    fi
    
    print_status "Account deployed successfully!"
    
else
    print_status "Using existing account"
fi

print_status "Account setup complete"

# Test account configuration
echo ""
print_info "Testing account configuration..."
if sncast account list > /dev/null 2>&1; then
    print_status "Account configuration is valid"
else
    print_warning "Could not validate account configuration, but continuing..."
fi

# Check network connectivity
echo ""
print_info "Checking network connectivity..."
CHAIN_ID=$(sncast show-config | grep chain_id | awk '{print $2}')
print_status "Connected to network: $CHAIN_ID"

# Continue with deployment
echo ""
print_info "Proceeding with deployment..."

# Get the contract artifact path
CONTRACT_ARTIFACT="target/dev/contracts_EventTestContract.contract_class.json"

if [ ! -f "$CONTRACT_ARTIFACT" ]; then
    print_error "Contract artifact not found: $CONTRACT_ARTIFACT"
    exit 1
fi

print_status "Contract artifact found: $CONTRACT_ARTIFACT"

# Final deployment confirmation
echo ""
print_warning "Final deployment details:"
echo "   Network: Mainnet"
echo "   Account: $ACCOUNT_FILE"
echo "   Contract: $CONTRACT_ARTIFACT"
echo "   RPC: $STARKNET_RPC"
echo "   Cost: Real STRK"
echo ""
read -p "Proceed with mainnet deployment? (yes/no): " final_confirm
if [ "$final_confirm" != "yes" ]; then
    print_info "Deployment cancelled"
    exit 0
fi

# Get account address and deployment status for logging
ACCOUNT_ADDRESS_RAW=$(sncast account list | grep "address:" | awk '{print $2}' | head -1)
ACCOUNT_DEPLOYED=$(sncast account list | grep "deployed:" | awk '{print $2}' | head -1)

# Ensure address is properly formatted to 66 characters (0x + 64 hex chars)
if [[ $ACCOUNT_ADDRESS_RAW =~ ^0x[0-9a-fA-F]+$ ]]; then
    # Remove 0x prefix, pad to 64 characters, then add 0x back
    HEX_PART=${ACCOUNT_ADDRESS_RAW#0x}
    PADDED_HEX=$(printf "%064s" "$HEX_PART" | tr ' ' '0')
    ACCOUNT_ADDRESS="0x$PADDED_HEX"
else
    ACCOUNT_ADDRESS="$ACCOUNT_ADDRESS_RAW"
fi

# Check if account is deployed
if [ "$ACCOUNT_DEPLOYED" = "false" ]; then
    print_error "Account is not deployed to mainnet yet!"
    echo "Account address: $ACCOUNT_ADDRESS"
    echo ""
    print_info "To deploy this account:"
    echo "1. Fund the account with STRK tokens: https://starkscan.co/contract/$ACCOUNT_ADDRESS"
    echo "2. Run: sncast account deploy --name oz_account --fee-token strk"
    echo "3. Then re-run this deployment script"
    exit 1
fi

# Declare the contract
echo ""
print_info "Declaring contract on mainnet..."
print_info "Using account address: $ACCOUNT_ADDRESS"
print_info "Account deployment status: $ACCOUNT_DEPLOYED"
print_warning "This may take several minutes and cost STRK..."
echo ""

# Run declare and capture both output and exit code
set +e  # Temporarily disable exit on error
DECLARE_OUTPUT=$(sncast declare --contract-name EventTestContract --fee-token strk 2>&1)
DECLARE_EXIT_CODE=$?
set -e  # Re-enable exit on error

# Log the output regardless of success/failure
echo "Declaration output:"
echo "$DECLARE_OUTPUT"
echo ""

if [ $DECLARE_EXIT_CODE -ne 0 ]; then
    print_error "Contract declaration failed with exit code: $DECLARE_EXIT_CODE"
    
    # Check for common error patterns and provide helpful messages
    if echo "$DECLARE_OUTPUT" | grep -q "transaction_hash"; then
        TRANSACTION_HASH=$(echo "$DECLARE_OUTPUT" | grep -o "transaction_hash: 0x[0-9a-fA-F]*" | awk '{print $2}')
        if [ ! -z "$TRANSACTION_HASH" ]; then
            print_info "Transaction was submitted: $TRANSACTION_HASH"
            print_info "Check status at: https://starkscan.co/tx/$TRANSACTION_HASH"
            print_info "Transaction may still be processing. Wait and check the explorer."
        fi
    fi
    
    if echo "$DECLARE_OUTPUT" | grep -q "already declared"; then
        print_warning "Contract class may already be declared. Checking for existing class hash..."
        # Try to extract class hash from error message if available
    fi
    
    exit 1
else
    print_status "Contract declaration submitted successfully!"
fi

# Extract class hash from declare output
CLASS_HASH=$(echo "$DECLARE_OUTPUT" | grep "class_hash:" | awk '{print $2}' | head -1)

if [ -z "$CLASS_HASH" ]; then
    print_error "Could not extract class hash from declare output"
    echo "$DECLARE_OUTPUT"
    exit 1
fi

print_status "Contract declared with class hash: $CLASS_HASH"

# Deploy the contract
echo ""
print_info "Deploying contract to mainnet..."
print_info "Using class hash: $CLASS_HASH"
print_warning "This may take several minutes and cost STRK..."
echo ""

# Run deploy and capture both output and exit code
set +e  # Temporarily disable exit on error
DEPLOY_OUTPUT=$(sncast deploy --class-hash "$CLASS_HASH" --fee-token strk 2>&1)
DEPLOY_EXIT_CODE=$?
set -e  # Re-enable exit on error

# Log the output regardless of success/failure
echo "Deployment output:"
echo "$DEPLOY_OUTPUT"
echo ""

if [ $DEPLOY_EXIT_CODE -ne 0 ]; then
    print_error "Contract deployment failed with exit code: $DEPLOY_EXIT_CODE"
    
    # Check for transaction hash even in failure cases
    if echo "$DEPLOY_OUTPUT" | grep -q "transaction_hash"; then
        TRANSACTION_HASH=$(echo "$DEPLOY_OUTPUT" | grep -o "transaction_hash: 0x[0-9a-fA-F]*" | awk '{print $2}')
        if [ ! -z "$TRANSACTION_HASH" ]; then
            print_info "Transaction was submitted: $TRANSACTION_HASH"
            print_info "Check status at: https://starkscan.co/tx/$TRANSACTION_HASH"
            print_info "Transaction may still be processing. Wait and check the explorer."
        fi
    fi
    
    exit 1
else
    print_status "Contract deployment submitted successfully!"
fi

# Extract contract address from deploy output
CONTRACT_ADDRESS=$(echo "$DEPLOY_OUTPUT" | grep -o "0x[0-9a-fA-F]\{64\}" | head -1)

if [ -z "$CONTRACT_ADDRESS" ]; then
    print_error "Could not extract contract address from deploy output"
    echo "$DEPLOY_OUTPUT"
    exit 1
fi

print_status "Contract deployed successfully!"
echo "Contract Address: $CONTRACT_ADDRESS"

# Save deployment info
DEPLOYMENT_INFO="deployment_info_$(date +%Y%m%d_%H%M%S).txt"
cat > "$DEPLOYMENT_INFO" << EOF
Starknet Mainnet Deployment
==========================
Date: $(date)
Network: Mainnet
Class Hash: $CLASS_HASH
Contract Address: $CONTRACT_ADDRESS
Account: $ACCOUNT_FILE
RPC: $STARKNET_RPC

CONTRACT ADDRESS FOR YOUR INDEXER: $CONTRACT_ADDRESS

Verification:
- Explorer: https://starkscan.co/contract/$CONTRACT_ADDRESS
- Class: https://starkscan.co/class/$CLASS_HASH
EOF

echo ""
print_status "Deployment info saved to: $DEPLOYMENT_INFO"

# Test the contract (optional)
echo ""
read -p "Do you want to test the contract on mainnet? (This will cost STRK) (yes/no): " test_confirm
if [ "$test_confirm" = "yes" ]; then
    print_info "Testing contract functions on mainnet..."
    print_info "Testing emit_basic_types_events..."

    TEST_OUTPUT=$(sncast invoke --contract-address "$CONTRACT_ADDRESS" --function emit_basic_types_events --fee-token strk 2>&1)

    if [ $? -eq 0 ]; then
        print_status "Basic types events test successful"
    else
        print_warning "Basic types events test failed"
        echo "$TEST_OUTPUT"
    fi
fi

# Final success message
echo ""
echo "🎉 DEPLOYMENT COMPLETE! 🎉"
echo "========================="
echo ""
print_info "NEXT STEPS:"
echo "1. Add this contract address to your indexer: $CONTRACT_ADDRESS"
echo "2. Verify on Starknet Explorer: https://starkscan.co/contract/$CONTRACT_ADDRESS"
echo "3. Test contract functions:"
echo "   sncast invoke --contract-address $CONTRACT_ADDRESS --function emit_all_events --fee-token strk"
echo ""
print_warning "CONTRACT ADDRESS: $CONTRACT_ADDRESS"
