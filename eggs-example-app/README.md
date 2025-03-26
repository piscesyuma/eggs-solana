# Eggs Example App

This is a simple dApp for interacting with the Eggs Solana program. It's built with Next.js, TypeScript, and TailwindCSS.

## Overview

The Eggs protocol allows users to:

1. Buy EGGS tokens with SOL
2. Sell EGGS tokens for SOL
3. Leverage trade EGGS using collateral

## Getting Started

### Prerequisites

- Node.js 16+ and npm
- A Solana wallet (Phantom, Solflare, Backpack, or Torus)

### Installation

```bash
# Install dependencies
npm install
```

### Running the App

```bash
# Start the development server
npm run dev
```

Visit [http://localhost:3000](http://localhost:3000) in your browser to see the app.

## Features

- Connect your Solana wallet
- View Eggs Protocol information
- Buy EGGS tokens with SOL
- Sell EGGS tokens for SOL  
- Leverage trade with EGGS
- View your active loans

## Technical Details

The app communicates with the Eggs Solana program using Anchor. The main components are:

- **WalletContextProvider**: Manages Solana wallet connection
- **AnchorClient**: Provides access to the Eggs program
- **EggsService**: Service layer for interacting with the program
- **UI Components**: React components for displaying info and forms

## Project Structure

```
/src
  /app - Next.js app directory
  /components - React components 
  /context - Context providers
  /idl - Program interface definition
  /services - Service layer for program interaction
  /utils - Utility functions
```

## Using the dApp

1. Connect your wallet using the wallet button
2. View protocol information (total minted, balances, etc.)
3. Buy EGGS with SOL or sell EGGS for SOL
4. Check the Leverage tab to see if you have active loans or create new ones

## Development

This is an example application. For production use, you would need to:

1. Add proper error handling
2. Implement robust account derivation
3. Add transaction confirmation tracking
4. Implement more security features

## License

This project is MIT licensed.
