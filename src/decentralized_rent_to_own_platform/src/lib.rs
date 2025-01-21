#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Enhanced Structs with more detailed information
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct LeaseContract {
    contract_id: u64,
    item_id: u64,
    owner_id: String,
    lessee_id: String,
    lease_terms: String,
    total_payments: u64,
    completed_payments: u64,
    monthly_payment: u64,
    lease_start: u64,
    lease_end: u64,
    is_active: bool,
    is_completed: bool,
    payment_history: Vec<PaymentRecord>,
    last_payment_date: Option<u64>,
    grace_period_days: u64,
    early_payoff_discount: Option<u64>,
    maintenance_records: Vec<MaintenanceRecord>,
    contract_status: ContractStatus,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct PaymentRecord {
    payment_id: u64,
    contract_id: u64,
    lessee_id: String,
    amount: u64,
    payment_date: u64,
    payment_type: PaymentType,
    transaction_hash: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct AssetNFT {
    item_id: u64,
    owner_id: String,
    description: String,
    value: u64,
    is_leased: bool,
    asset_condition: AssetCondition,
    maintenance_history: Vec<MaintenanceRecord>,
    warranty_info: Option<String>,
    asset_type: AssetType,
    metadata: AssetMetadata,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Debug)]
enum ContractStatus {
    Active,
    Completed,
    Defaulted,
    Terminated,
    PendingApproval,
}

impl Default for ContractStatus {
    fn default() -> Self {
        ContractStatus::PendingApproval
    }
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
enum PaymentType {
    Regular,
    EarlyPayoff,
    LatePayment,
    SecurityDeposit,
}

impl Default for PaymentType {
    fn default() -> Self {
        PaymentType::Regular
    }
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
enum AssetCondition {
    Excellent,
    Good,
    Fair,
    Poor,
    NeedsRepair,
}

impl Default for AssetCondition {
    fn default() -> Self {
        AssetCondition::Good
    }
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct MaintenanceRecord {
    record_id: u64,
    item_id: u64,
    service_date: u64,
    description: String,
    cost: u64,
    performed_by: String,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
enum AssetType {
    Vehicle,
    Electronics,
    Furniture,
    Appliance,
    Other,
}

impl Default for AssetType {
    fn default() -> Self {
        AssetType::Other
    }
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Debug)]
enum ContractError {
    NotFound(String),
    AccessDenied(String),
    InvalidState(String),
    SystemError(String),
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct AssetMetadata {
    manufacturer: String,
    model: String,
    year: u64,
    serial_number: Option<String>,
    additional_details: Vec<(String, String)>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct LeaseContractDetails {
    contract: LeaseContract,
    days_remaining: u64,
    payment_status: PaymentStatus,
    next_payment_date: Option<u64>,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
enum PaymentStatus {
    Current,
    Late(u64),        // days late
    GracePeriod(u64), // days remaining in grace period
    Defaulted,
}

// Implement Storable for LeaseContract
impl Storable for LeaseContract {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for LeaseContract {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Implement Storable for AssetNFT
impl Storable for AssetNFT {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for AssetNFT {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl std::fmt::Display for ContractError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ContractError::NotFound(msg) => write!(f, "Contract not found: {}", msg),
            ContractError::AccessDenied(msg) => write!(f, "Access denied: {}", msg),
            ContractError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            ContractError::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

// Thread-local storage
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static LEASE_CONTRACTS: RefCell<StableBTreeMap<u64, LeaseContract, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static ASSETS: RefCell<StableBTreeMap<u64, AssetNFT, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));
}

/**
 * Payloads for the canister methods
 */

// LeaseContract Payload
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct CreateLeaseContractPayload {
    item_id: u64,
    lessee_id: String,
    lease_terms: String,
    total_payments: u64,
    monthly_payment: u64,
    lease_duration: u64,
    grace_period_days: u64,
    early_payoff_discount: Option<u64>,
}

// Create AssetNFT Payload
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct AssetPayload {
    description: String,
    value: u64,
    asset_condition: AssetCondition,
    asset_type: AssetType,
    metadata: AssetMetadata,
    warranty_info: Option<String>,
}

// Make payment Payload
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct MakePaymentPayload {
    contract_id: u64,
    amount: u64,
    payment_type: PaymentType,
    transaction_hash: String,
}

// Add maintenance record Payload
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct AddMaintenanceRecordPayload {
    item_id: u64,
    description: String,
    cost: u64,
}

// Terminate contract Payload
#[derive(candid::CandidType, Clone, Serialize, Deserialize)]
struct TerminateContractPayload {
    contract_id: u64,
    reason: String,
}

//  Helper functions for contract details
fn calculate_days_remaining(contract: &LeaseContract) -> u64 {
    let current_time = time();
    if current_time >= contract.lease_end {
        0
    } else {
        (contract.lease_end - current_time) / (24 * 60 * 60 * 1_000_000_000)
    }
}

fn check_payment_status(contract: &LeaseContract) -> PaymentStatus {
    let current_time = time();

    if let Some(last_payment) = contract.last_payment_date {
        let days_since_payment = (current_time - last_payment) / (24 * 60 * 60 * 1_000_000_000);
        let payment_due_days = 30; // Assuming monthly payments

        if days_since_payment <= payment_due_days {
            PaymentStatus::Current
        } else if days_since_payment <= payment_due_days + contract.grace_period_days {
            PaymentStatus::GracePeriod(
                payment_due_days + contract.grace_period_days - days_since_payment,
            )
        } else if days_since_payment <= payment_due_days + 30 {
            PaymentStatus::Late(days_since_payment - payment_due_days)
        } else {
            PaymentStatus::Defaulted
        }
    } else {
        PaymentStatus::Current // For new contracts
    }
}

fn calculate_next_payment_date(contract: &LeaseContract) -> Option<u64> {
    if contract.is_completed || !contract.is_active {
        None
    } else if let Some(last_payment) = contract.last_payment_date {
        Some(last_payment + 30 * 24 * 60 * 60 * 1_000_000_000) // 30 days in nanoseconds
    } else {
        Some(contract.lease_start + 30 * 24 * 60 * 60 * 1_000_000_000)
    }
}

/**
 * Canister methods
 *
 */

/// Function to validate an AssetPayload
fn validate_asset_payload(payload: &AssetPayload) -> Result<(), String> {
    if payload.description.trim().is_empty() {
        return Err("Description cannot be empty.".to_string());
    }
    if payload.value == 0 {
        return Err("Value must be greater than 0.".to_string());
    }
    if payload.asset_condition.trim().is_empty() {
        return Err("Asset condition cannot be empty.".to_string());
    }
    if payload.asset_type.trim().is_empty() {
        return Err("Asset type cannot be empty.".to_string());
    }
    Ok(())
}


// Create AssetNFT
#[ic_cdk::update]
fn create_asset_nft(payload: AssetPayload) -> Result<AssetNFT, String> {
    // Validate the payload
    validate_asset_payload(&payload)?;

    // Generate a unique ID for the new asset
    let item_id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Failed to generate asset ID");

    // Create a new asset
    let asset = AssetNFT {
        item_id,
        owner_id: ic_cdk::caller().to_string(),
        description: payload.description,
        value: payload.value,
        is_leased: false,
        asset_condition: payload.asset_condition,
        maintenance_history: Vec::new(),
        warranty_info: payload.warranty_info,
        asset_type: payload.asset_type,
        metadata: payload.metadata,
    };

    // Store the asset in the stable map
    ASSETS.with(|assets| assets.borrow_mut().insert(item_id, asset.clone()));
    Ok(asset)
}


//
#[ic_cdk::query]
fn get_asset_details(item_id: u64) -> Result<AssetNFT, String> {
    ASSETS.with(|assets| {
        match assets.borrow().get(&item_id) {
            Some(asset) => Ok(asset.clone()), // Return the found asset
            None => Err(format!("Asset with ID {} not found", item_id)), // Return an error message
        }
    })
}

/// Function to validate a LeasePayload
fn validate_lease_payload(payload: &LeasePayload) -> Result<(), String> {
    if payload.asset_id == 0 {
        return Err("Asset ID must be greater than 0.".to_string());
    }
    if payload.lessee_id.trim().is_empty() {
        return Err("Lessee ID cannot be empty.".to_string());
    }
    if payload.lease_duration == 0 {
        return Err("Lease duration must be greater than 0.".to_string());
    }
    if payload.lease_start_time == 0 {
        return Err("Lease start time must be provided.".to_string());
    }
    Ok(())
}


// Create lease contract
#[ic_cdk::update]
fn create_lease_contract(payload: LeasePayload) -> Result<LeaseContract, String> {
    // Validate the payload
    validate_lease_payload(&payload)?;

    // Generate a unique ID for the new lease contract
    let contract_id = LEASE_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Failed to generate contract ID");

    // Calculate the lease end time
    let lease_end_time = payload.lease_start_time + payload.lease_duration;

    // Create the lease contract
    let lease = LeaseContract {
        contract_id,
        asset_id: payload.asset_id,
        lessor_id: ic_cdk::caller().to_string(),
        lessee_id: payload.lessee_id,
        lease_duration: payload.lease_duration,
        lease_start_time: payload.lease_start_time,
        lease_end_time,
    };

    // Store the lease in the stable map
    LEASES.with(|leases| leases.borrow_mut().insert(contract_id, lease.clone()));
    Ok(lease)
}

// Fetch lease contract by id
#[ic_cdk::query]
fn get_lease_contract(contract_id: u64) -> Result<LeaseContract, ContractError> {
    let caller = ic_cdk::caller().to_string();

    LEASE_CONTRACTS
        .with(|contracts| {
            let contracts_ref = contracts.borrow();

            // Attempt to get the contract
            match contracts_ref.get(&contract_id) {
                Some(contract) => {
                    // Verify access permissions
                    if contract.owner_id == caller || contract.lessee_id == caller {
                        Ok(contract)
                    } else {
                        Err(ContractError::AccessDenied(format!(
                            "Caller {} is neither owner nor lessee of contract {}",
                            caller, contract_id
                        )))
                    }
                }
                None => Err(ContractError::NotFound(format!(
                    "Contract with ID {} does not exist",
                    contract_id
                ))),
            }
        })
        .map_err(|e| {
            ContractError::SystemError(format!("Failed to access contract storage: {}", e))
        })
}

// Optional helper function to handle specific error cases
#[ic_cdk::query]
fn get_lease_contract_details(contract_id: u64) -> Result<LeaseContractDetails, ContractError> {
    match get_lease_contract(contract_id) {
        Ok(contract) => {
            // Check contract status and provide appropriate information
            if !contract.is_active {
                return Err(ContractError::InvalidState(format!(
                    "Contract {} is no longer active. Status: {:?}",
                    contract_id, contract.contract_status
                )));
            }

            // Create a detailed view of the contract
            Ok(LeaseContractDetails {
                contract: contract.clone(),
                days_remaining: calculate_days_remaining(&contract),
                payment_status: check_payment_status(&contract),
                next_payment_date: calculate_next_payment_date(&contract),
            })
        }
        Err(e) => Err(e),
    }
}

#[ic_cdk::update]
fn make_payment(payload: MakePaymentPayload) -> Result<LeaseContract, String> {
    LEASE_CONTRACTS.with(|contracts| {
        let mut contracts = contracts.borrow_mut();
        let lease = contracts.get(&payload.contract_id);

        if let Some(mut lease) = lease {
            if lease.is_completed {
                return Err("Lease is already completed".to_string());
            }

            // Validate payment amount based on payment type
            match payload.payment_type {
                PaymentType::Regular => {
                    if payload.amount < lease.monthly_payment {
                        return Err("Payment amount is less than required".to_string());
                    }
                }
                PaymentType::EarlyPayoff => {
                    let remaining_balance =
                        (lease.total_payments - lease.completed_payments) * lease.monthly_payment;
                    let required_amount = if let Some(discount) = lease.early_payoff_discount {
                        remaining_balance - (remaining_balance * discount / 100)
                    } else {
                        remaining_balance
                    };
                    if payload.amount < required_amount {
                        return Err(format!("Early payoff requires {}", required_amount));
                    }
                    lease.is_completed = true;
                    lease.is_active = false;
                    lease.contract_status = ContractStatus::Completed;
                }
                _ => {}
            }

            // Record payment
            let payment_record = PaymentRecord {
                payment_id: lease.payment_history.len() as u64,
                contract_id: payload.contract_id,
                lessee_id: lease.lessee_id.clone(),
                amount: payload.amount,
                payment_date: time(),
                payment_type: payload.payment_type,
                transaction_hash: payload.transaction_hash,
            };

            lease.payment_history.push(payment_record);
            lease.last_payment_date = Some(time());
            lease.completed_payments += 1;

            // Check if lease is completed through regular payments
            if lease.completed_payments >= lease.total_payments {
                lease.is_completed = true;
                lease.is_active = false;
                lease.contract_status = ContractStatus::Completed;
            }

            contracts.insert(payload.contract_id, lease.clone());
            Ok(lease)
        } else {
            Err("Lease contract not found".to_string())
        }
    })
}

#[ic_cdk::update]
fn add_maintenance_record(
    payload: AddMaintenanceRecordPayload,
) -> Result<MaintenanceRecord, String> {
    ASSETS.with(|assets| {
        let mut assets = assets.borrow_mut();
        if let Some(mut asset) = assets.get(&payload.item_id) {
            let record = MaintenanceRecord {
                record_id: asset.maintenance_history.len() as u64,
                item_id: payload.item_id,
                service_date: time(),
                description: payload.description,
                cost: payload.cost,
                performed_by: ic_cdk::caller().to_string(),
            };
            asset.maintenance_history.push(record.clone());

            assets.insert(payload.item_id, asset);

            Ok(record)
        } else {
            Err("Asset not found".to_string())
        }
    })
}

#[ic_cdk::query]
fn get_payment_history(contract_id: u64) -> Result<Vec<PaymentRecord>, String> {
    LEASE_CONTRACTS.with(|contracts| {
        if let Some(contract) = contracts.borrow().get(&contract_id) {
            Ok(contract.payment_history)
        } else {
            Err("Contract not found".to_string())
        }
    })
}

#[ic_cdk::update]
fn terminate_contract(payload: TerminateContractPayload) -> Result<LeaseContract, String> {
    LEASE_CONTRACTS.with(|contracts| {
        let mut contracts = contracts.borrow_mut();
        if let Some(mut contract) = contracts.get(&payload.contract_id) {
            if !contract.is_active {
                return Err("Contract is not active".to_string());
            }
            contract.is_active = false;
            contract.contract_status = ContractStatus::Terminated;
            contracts.insert(payload.contract_id, contract.clone());
            Ok(contract)
        } else {
            Err("Contract not found".to_string())
        }
    })
}

ic_cdk::export_candid!();
