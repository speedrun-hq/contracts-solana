# Solana Cross-Chain Transfer Intents for ZetaChain

This is a Solana program for creating cross-chain transfer intents using the ZetaChain interoperability protocol. The program allows users to deposit tokens on Solana for transfer to a separate chain identified by a chain ID.

## Overview

This program implements a system for creating transfer intents that will be processed by ZetaChain's cross-chain protocol. Here's how it works:

1. A user initiates a transfer by depositing tokens into the program
2. The program generates a unique intent ID and passes the token along with necessary information to the ZetaChain gateway
3. The ZetaChain protocol will then handle the cross-chain transfer to the recipient on the target chain

## Key Features

- Generate unique intent IDs for cross-chain transfers
- Transfer SPL tokens to different blockchains through ZetaChain
- Include tips for fulfillers of cross-chain transfers
- Integrate with ZetaChain's gateway contract

## Program Instructions

### Initialize

Initializes the program state with the gateway address and router address.

### Initiate Intent

Creates a new cross-chain transfer intent with the following parameters:
- `amount`: The amount of tokens to transfer
- `target_chain`: The destination chain ID
- `receiver`: The recipient address on the target chain
- `tip`: Additional amount to incentivize fulfillers
- `salt`: Random value for intent ID generation

### Get Next Intent ID

Calculates the next intent ID that would be generated with the current counter and provided salt.

## Building and Testing

This project uses the Anchor framework for Solana development.

```bash
# Install dependencies
npm install

# Build the program
anchor build

# Run tests
anchor test
```

## Integration with ZetaChain

The program interacts with ZetaChain through the gateway's `deposit_spl_token_and_call` function, which is responsible for handling the cross-chain messaging and token transfers.

## License

ISC 