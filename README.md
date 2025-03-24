# EGGS Token Program

A Solana program implementing the EGGS token and its associated functionality, converted from an Ethereum Solidity contract.

## Overview

The EGGS program is a Solana-based implementation of a token system with the following features:

- EGGS token creation and management
- Buy/sell mechanisms with configurable fees
- Lending and borrowing functionality
- Leverage features
- Liquidation mechanisms
- Various utility functions for token economics

## Project Structure

- `programs/eggs/src/`: Contains the core program logic
  - `lib.rs`: Main program implementation with instruction handlers
  - `states.rs`: State definitions for the program accounts

## Getting Started

### Prerequisites

- Solana CLI
- Anchor Framework
- Node.js and NPM/Yarn

### Installation

1. Clone the repository
2. Install dependencies: `yarn install`

### Building

```bash
anchor build
```

### Testing

```bash
anchor test
```

## Program Instructions

The program provides the following instructions:

- `initialize`: Initialize the EGGS program with a new token mint
- `setFeeAddress`: Set the fee address for the program
- `setStart`: Start trading by providing initial liquidity
- `setBuyFee`: Set the buy fee percentage
- `setBuyFeeLeverage`: Set the buy fee leverage percentage
- `setSellFee`: Set the sell fee percentage
- `buy`: Buy EGGS tokens with SOL
- `sell`: Sell EGGS tokens for SOL

Additional instructions for lending, borrowing, and leverage functionality:

- `leverage`: Borrow SOL using EGGS as collateral with leverage
- `borrow`: Borrow SOL using EGGS as collateral
- `borrowMore`: Borrow additional SOL from an existing loan
- `removeCollateral`: Remove excess collateral from a loan
- `repay`: Repay part of a loan
- `closePosition`: Close a loan position by repaying in full
- `flashClosePosition`: Close a position by using the collateral to repay
- `extendLoan`: Extend the duration of a loan
- `liquidate`: Liquidate expired loans

## License

BUSL-1.1 (Business Source License 1.1) 