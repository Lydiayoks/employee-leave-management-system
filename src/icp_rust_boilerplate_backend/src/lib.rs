#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::fmt;
use std::{borrow::Cow, cell::RefCell, collections::HashMap};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Employee {
    id: u64,
    name: String,
    email: String,
    leave_balances: HashMap<LeaveName, u32>, // Balances per leave type
    created_at: u64,
}

#[derive(candid::CandidType, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
enum LeaveName {
    #[default]
    Annual,
    Sick,
    Maternity,
    Paternity,
    Unpaid,
}

impl fmt::Display for LeaveName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let leave_name_str = match self {
            LeaveName::Annual => "Annual",
            LeaveName::Sick => "Sick",
            LeaveName::Maternity => "Maternity",
            LeaveName::Paternity => "Paternity",
            LeaveName::Unpaid => "Unpaid",
        };
        write!(f, "{}", leave_name_str)
    }
}

#[derive(candid::CandidType, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
enum LeaveStatusEnum {
    #[default]
    Pending,
    Approved,
    Rejected,
    Accrued,
    Canceled,
}

impl fmt::Display for LeaveStatusEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            LeaveStatusEnum::Pending => "Pending",
            LeaveStatusEnum::Approved => "Approved",
            LeaveStatusEnum::Rejected => "Rejected",
            LeaveStatusEnum::Accrued => "Accrued",
            LeaveStatusEnum::Canceled => "Canceled",
        };
        write!(f, "{}", status_str)
    }
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default, Debug)]
struct LeaveRequest {
    id: u64,
    employee_id: u64,
    leave_type_id: u64,
    start_date: u64,
    end_date: u64,
    status: LeaveStatusEnum,
    reason: String,
    created_at: u64,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct LeaveType {
    id: u64,
    name: LeaveName,
    quota: u32, // Max days allowed per year
    carryover_allowed: bool,
    created_at: u64,
}

impl Storable for Employee {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Employee {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for LeaveRequest {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for LeaveRequest {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for LeaveType {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for LeaveType {
    const MAX_SIZE: u32 = 512;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static EMPLOYEE_STORAGE: RefCell<StableBTreeMap<u64, Employee, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static LEAVE_REQUESTS_STORAGE: RefCell<StableBTreeMap<u64, LeaveRequest, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static LEAVE_TYPES_STORAGE: RefCell<StableBTreeMap<u64, LeaveType, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct EmployeePayload {
    name: String,
    email: String,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct LeaveRequestPayload {
    employee_id: u64,
    leave_type_id: u64,
    start_date: u64,
    end_date: u64,
    reason: String,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
struct LeaveTypePayload {
    name: LeaveName,
    quota: u32,
    carryover_allowed: bool,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Message {
    Success(String),
    Error(String),
    NotFound(String),
    InvalidPayload(String),
}

// Function to safely increment the ID counter and handle any potential errors
fn increment_id_counter() -> Result<u64, Message> {
    ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow().get();
        counter.borrow_mut().set(current_value + 1)
            .map_err(|_| Message::Error("Failed to increment ID counter.".to_string()))?;
        Ok(current_value + 1)
    })
}

// Function to fetch an employee by ID with error handling
fn get_employee_by_id(employee_id: u64) -> Result<Employee, Message> {
    EMPLOYEE_STORAGE.with(|storage| {
        storage
            .borrow()
            .get(&employee_id)
            .ok_or_else(|| Message::NotFound(format!("Employee with id={} not found.", employee_id)))
    })
}

// Function to fetch a leave type by ID with error handling
fn get_leave_type_by_id(leave_type_id: u64) -> Result<LeaveType, Message> {
    LEAVE_TYPES_STORAGE.with(|storage| {
        storage
            .borrow()
            .get(&leave_type_id)
            .ok_or_else(|| Message::NotFound(format!("Leave type with id={} not found.", leave_type_id)))
    })
}

// Function to fetch a leave request by ID with error handling
fn get_leave_request_by_id(request_id: u64) -> Result<LeaveRequest, Message> {
    LEAVE_REQUESTS_STORAGE.with(|storage| {
        storage
            .borrow()
            .get(&request_id)
            .ok_or_else(|| Message::NotFound(format!("Leave request with id={} not found.", request_id)))
    })
}

// Create a new employee
#[ic_cdk::update]
fn create_employee(payload: EmployeePayload) -> Result<Employee, Message> {
    if payload.name.is_empty() || payload.email.is_empty() {
        return Err(Message::InvalidPayload("Ensure 'name' and 'email' are provided.".to_string()));
    }

    let id = increment_id_counter()?;
    let employee = Employee {
        id,
        name: payload.name,
        email: payload.email,
        leave_balances: vec![
            (LeaveName::Annual, 20),
            (LeaveName::Sick, 20),
            (LeaveName::Maternity, 20),
            (LeaveName::Paternity, 20),
            (LeaveName::Unpaid, 20),
        ]
        .into_iter()
        .collect(),
        created_at: current_time(),
    };

    EMPLOYEE_STORAGE.with(|storage| storage.borrow_mut().insert(id, employee.clone()));
    Ok(employee)
}

// New Function: Search for an employee by name or email
#[ic_cdk::query]
fn search_employee(query: String) -> Result<Vec<Employee>, Message> {
    let results: Vec<Employee> = EMPLOYEE_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .filter(|(_, employee)| {
                employee.name.to_lowercase().contains(&query.to_lowercase())
                    || employee.email.to_lowercase().contains(&query.to_lowercase())
            })
            .map(|(_, employee)| employee.clone())
            .collect()
    });

    if results.is_empty() {
        Err(Message::NotFound(format!("No employees found matching '{}'.", query)))
    } else {
        Ok(results)
    }
}

// New Function: Update the leave balances for an employee
#[ic_cdk::update]
fn update_leave_balance(employee_id: u64, leave_name: LeaveName, new_balance: u32) -> Result<Message, Message> {
    let mut employee = get_employee_by_id(employee_id)?;

    // Update the leave balance for the specified leave type
    employee.leave_balances.insert(leave_name, new_balance);

    EMPLOYEE_STORAGE.with(|storage| storage.borrow_mut().insert(employee_id, employee));

    Ok(Message::Success(format!(
        "Leave balance for '{}' updated successfully.",
        leave_name
    )))
}

// New Function: Get detailed leave history for an employee
#[ic_cdk::query]
fn get_leave_history(employee_id: u64) -> Result<Vec<LeaveRequest>, Message> {
    let requests: Vec<LeaveRequest> = LEAVE_REQUESTS_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .filter(|(_, request)| request.employee_id == employee_id)
            .map(|(_, request)| request.clone())
            .collect()
    });

    if requests.is_empty() {
        Err(Message::NotFound("No leave requests found for this employee.".to_string()))
    } else {
        Ok(requests)
    }
}

// Get all employees
#[ic_cdk::query]
fn get_employees() -> Result<Vec<Employee>, Message> {
    EMPLOYEE_STORAGE.with(|storage| {
        let employees: Vec<Employee> = storage
            .borrow()
            .iter()
            .map(|(_, employee)| employee.clone())
            .collect();

        if employees.is_empty() {
            Err(Message::NotFound("No employees found".to_string()))
        } else {
            Ok(employees)
        }
    })
}

// Create a leave request
#[ic_cdk::update]
fn create_leave_request(payload: LeaveRequestPayload) -> Result<LeaveRequest, Message> {
    if payload.reason.is_empty() {
        return Err(Message::InvalidPayload("Ensure 'reason' is provided.".to_string()));
    }

    let leave_type = get_leave_type_by_id(payload.leave_type_id)?;
    let employee = get_employee_by_id(payload.employee_id)?;

    let id = increment_id_counter()?;

    let leave_request = LeaveRequest {
        id,
        employee_id: employee.id,
        leave_type_id: leave_type.id,
        start_date: payload.start_date,
        end_date: payload.end_date,
        status: LeaveStatusEnum::Pending,
        reason: payload.reason,
        created_at: current_time(),
    };

    LEAVE_REQUESTS_STORAGE.with(|storage| storage.borrow_mut().insert(id, leave_request.clone()));
    Ok(leave_request)
}

// Get all leave requests
#[ic_cdk::query]
fn get_leave_requests() -> Result<Vec<LeaveRequest>, Message> {
    LEAVE_REQUESTS_STORAGE.with(|storage| {
        let requests: Vec<LeaveRequest> = storage
            .borrow()
            .iter()
            .map(|(_, request)| request.clone())
            .collect();

        if requests.is_empty() {
            Err(Message::NotFound("No leave requests found".to_string()))
        } else {
            Ok(requests)
        }
    })
}

// Approve a leave request
#[ic_cdk::update]
fn approve_leave_request(request_id: u64) -> Result<Message, Message> {
    let mut request = get_leave_request_by_id(request_id)?;

    if request.status == LeaveStatusEnum::Approved {
        return Err(Message::Error("Leave request is already approved.".to_string()));
    }

    let mut employee = get_employee_by_id(request.employee_id)?;
    let leave_type = get_leave_type_by_id(request.leave_type_id)?;

    let balance = employee
        .leave_balances
        .entry(leave_type.name.clone())
        .or_insert(0);

    if *balance < leave_type.quota {
        return Err(Message::Error(format!(
            "Insufficient balance for {} leave. Current balance: {}",
            leave_type.name, *balance
        )));
    }

    *balance = balance.saturating_sub(leave_type.quota);

    request.status = LeaveStatusEnum::Approved;
    LEAVE_REQUESTS_STORAGE.with(|storage| storage.borrow_mut().insert(request_id, request));
    EMPLOYEE_STORAGE.with(|storage| storage.borrow_mut().insert(employee.id, employee));

    Ok(Message::Success("Leave request approved and balance updated.".to_string()))
}

// Reject a leave request
#[ic_cdk::update]
fn reject_leave_request(request_id: u64) -> Result<Message, Message> {
    let mut request = get_leave_request_by_id(request_id)?;

    if request.status == LeaveStatusEnum::Rejected {
        return Err(Message::Error("Leave request is already rejected.".to_string()));
    }

    request.status = LeaveStatusEnum::Rejected;
    LEAVE_REQUESTS_STORAGE.with(|storage| storage.borrow_mut().insert(request_id, request));

    Ok(Message::Success("Leave request rejected.".to_string()))
}

// Cancel a leave request
#[ic_cdk::update]
fn cancel_leave_request(request_id: u64) -> Result<Message, Message> {
    let mut request = get_leave_request_by_id(request_id)?;

    if request.status == LeaveStatusEnum::Approved {
        return Err(Message::Error("Cannot cancel an approved leave request.".to_string()));
    }

    request.status = LeaveStatusEnum::Canceled;
    LEAVE_REQUESTS_STORAGE.with(|storage| storage.borrow_mut().insert(request_id, request));

    Ok(Message::Success("Leave request canceled.".to_string()))
}

// Create a new leave type
#[ic_cdk::update]
fn create_leave_type(payload: LeaveTypePayload) -> Result<LeaveType, Message> {
    let id = increment_id_counter()?;
    let leave_type = LeaveType {
        id,
        name: payload.name,
        quota: payload.quota,
        carryover_allowed: payload.carryover_allowed,
        created_at: current_time(),
    };

    LEAVE_TYPES_STORAGE.with(|storage| storage.borrow_mut().insert(id, leave_type.clone()));
    Ok(leave_type)
}

// Get all leave types
#[ic_cdk::query]
fn get_leave_types() -> Result<Vec<LeaveType>, Message> {
    LEAVE_TYPES_STORAGE.with(|storage| {
        let leave_types: Vec<LeaveType> = storage
            .borrow()
            .iter()
            .map(|(_, leave_type)| leave_type.clone())
            .collect();

        if leave_types.is_empty() {
            Err(Message::NotFound("No leave types found".to_string()))
        } else {
            Ok(leave_types)
        }
    })
}

// Accrue a leave request (finalize and update balances)
#[ic_cdk::update]
fn accrue_leave(leave_request_id: u64) -> Result<Message, Message> {
    let mut leave_request = get_leave_request_by_id(leave_request_id)?;

    if leave_request.status == LeaveStatusEnum::Accrued {
        return Err(Message::Error("Leave request already accrued.".to_string()));
    }

    if leave_request.status != LeaveStatusEnum::Approved {
        return Err(Message::Error("Leave request is not approved.".to_string()));
    }

    let mut employee = get_employee_by_id(leave_request.employee_id)?;
    let leave_type = get_leave_type_by_id(leave_request.leave_type_id)?;

    let balance = employee
        .leave_balances
        .entry(leave_type.name.clone())
        .or_insert(0);

    *balance = balance.saturating_sub(leave_type.quota);

    leave_request.status = LeaveStatusEnum::Accrued;

    LEAVE_REQUESTS_STORAGE
        .with(|storage| storage.borrow_mut().insert(leave_request_id, leave_request));

    EMPLOYEE_STORAGE.with(|storage| storage.borrow_mut().insert(employee.id, employee));

    Ok(Message::Success(
        "Leave request accrued successfully and balance updated.".to_string(),
    ))
}

// Generate a detailed leave report for an employee
#[ic_cdk::query]
fn generate_leave_report(employee_id: u64) -> Result<String, Message> {
    let leave_requests = LEAVE_REQUESTS_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .filter(|(_, request)| request.employee_id == employee_id)
            .map(|(_, request)| request.clone())
            .collect::<Vec<LeaveRequest>>()
    });

    let employee = get_employee_by_id(employee_id)?;

    let mut report = format!(
        "Leave report for {} (Employee ID: {})\n\n",
        employee.name, employee.id
    );

    for request in leave_requests {
        let leave_type = get_leave_type_by_id(request.leave_type_id)?;

        let leave_balance = employee.leave_balances.get(&leave_type.name).unwrap_or(&0);
        let remaining_leaves = *leave_balance;

        report += &format!(
            "Leave Request ID: {}\nLeave Type: {}\nQuota: {}\nRemaining Leaves: {}\nStart Date: {}\nEnd Date: {}\nStatus: {}\nReason: {}\n\n",
            request.id,
            leave_type.name,
            leave_type.quota,
            remaining_leaves,
            request.start_date,
            request.end_date,
            request.status, // Now correctly uses the Display trait
            request.reason
        );
    }

    Ok(report)
}

fn current_time() -> u64 {
    time()
}

// Export the candid interface for the canister
ic_cdk::export_candid!();