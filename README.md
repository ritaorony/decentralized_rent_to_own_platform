# Decentralized Rent-to-Own Platform

This project is a **Decentralized Rent-to-Own Platform** that allows users to lease items or property with an option to own after completing payments. It leverages **Web3 technology**, **smart contracts**, and **stable storage** for secure and transparent transactions.

## Features

- **Lease Contracts:** Users can create and manage lease agreements on-chain.
- **Asset Ownership:** Tracks asset ownership and lease status.
- **Payments & Transactions:** Handles monthly payments, early payoff discounts, and payment history.
- **Maintenance Records:** Logs maintenance activities for leased assets.
- **Smart Contract Enforcement:** Ensures contract rules are adhered to through immutable smart contracts.
- **Secure Storage:** Utilizes **StableBTreeMap** for efficient and persistent storage of lease contracts and assets.

## Tech Stack

- **Rust**: Primary language for smart contract logic.
- **IC Canisters**: Deployed on the Internet Computer for decentralized storage and execution.
- **Candid**: Interface definition language for defining methods and data types.
- **Stable Structures**: Ensures contract data remains persistent across updates.

## Smart Contract Components

### Data Structures

1. **LeaseContract** - Represents an active lease agreement.
2. **PaymentRecord** - Logs all payments made towards a lease.
3. **AssetNFT** - Represents an asset available for lease.
4. **ContractStatus** - Enum for tracking contract state (Active, Completed, Defaulted, etc.).
5. **PaymentType** - Enum defining payment categories (Regular, Late, EarlyPayoff, etc.).
6. **AssetCondition** - Enum indicating the asset's condition.
7. **MaintenanceRecord** - Stores maintenance history of assets.
8. **AssetType** - Enum defining asset categories (Vehicle, Electronics, Furniture, etc.).
9. **ContractError** - Enum for handling contract-related errors.

### Smart Contract Functions

#### Asset Management

- `create_asset_nft(payload: AssetPayload) -> Result<AssetNFT, String>`: Creates an asset NFT.
- `get_asset_details(item_id: u64) -> Result<AssetNFT, String>`: Retrieves asset details.
- `add_maintenance_record(payload: AddMaintenanceRecordPayload) -> Result<MaintenanceRecord, String>`: Adds maintenance records to an asset.

#### Lease Contracts

- `create_lease_contract(payload: CreateLeaseContractPayload) -> Result<LeaseContract, String>`: Creates a lease contract for an asset.
- `get_lease_contract(contract_id: u64) -> Result<LeaseContract, ContractError>`: Retrieves a lease contract.
- `get_lease_contract_details(contract_id: u64) -> Result<LeaseContractDetails, ContractError>`: Retrieves detailed contract information.
- `terminate_contract(payload: TerminateContractPayload) -> Result<LeaseContract, String>`: Terminates a lease contract.

#### Payments

- `make_payment(payload: MakePaymentPayload) -> Result<LeaseContract, String>`: Processes payments towards lease contracts.
- `get_payment_history(contract_id: u64) -> Result<Vec<PaymentRecord>, String>`: Retrieves payment history for a lease contract.

### Helper Functions

- `calculate_days_remaining(contract: &LeaseContract) -> u64`: Calculates remaining days in the lease term.
- `check_payment_status(contract: &LeaseContract) -> PaymentStatus`: Determines if a lease is **current, late, in grace period, or defaulted**.
- `calculate_next_payment_date(contract: &LeaseContract) -> Option<u64>`: Computes the next payment due date.

## 📋 Prerequisites

- Rust (latest stable version)
- Internet Computer SDK (DFX)
- Node.js and npm (for frontend development)
- Cargo (Rust package manager)

## 🛠 Installation

1. Clone the repository:

```bash
git clone https://github.com/ritaorony/decentralized_rent_to_own_platform.git
cd decentralized_rent_to_own_platform
```

2. Start the local Internet Computer replica:

```bash
dfx start --background
```

3. Deploy the canister:

```bash
npm run gen-deploy
```