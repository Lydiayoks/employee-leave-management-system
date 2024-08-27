#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::fmt;
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Employee {
    id: u64,
    name: String,
    email: String,
    leave_balances: std::collections::HashMap<LeaveName, u32>, // Balances per leave type
    created_at: u64,
}

// LeaveName enum
#[derive(
    candid::CandidType, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default, Debug,
)]

enum LeaveName {
    #[default]
    Annual,
    Sick,
    Maternity,
    Paternity,
    Unpaid,
}

// Implementing the Display trait for the LeaveName enum
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

// LeaveStatus enum
#[derive(
    candid::CandidType, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Default, Debug,
)]
enum LeaveStatusEnum {
    #[default]
    Pending,
    Approved,
    Rejected,
}

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default, Debug)]
struct LeaveRequest {
    id: u64,
    employee_id: u64,
    leave_type_id: u64,
    start_date: u64,
    end_date: u64,
    status: String,
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

#[ic_cdk::update]
fn create_employee(payload: EmployeePayload) -> Result<Employee, Message> {
    if payload.name.is_empty() || payload.email.is_empty() {
        return Err(Message::InvalidPayload(
            "Ensure 'name' and 'email' are provided.".to_string(),
        ));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let employee = Employee {
        id,
        name: payload.name,
        email: payload.email,
        // Initialize the leave balances for the employee to 20 days for each leave type
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

#[ic_cdk::update]
fn create_leave_request(payload: LeaveRequestPayload) -> Result<LeaveRequest, Message> {
    if payload.reason.is_empty() {
        return Err(Message::InvalidPayload(
            "Ensure 'reason' is provided.".to_string(),
        ));
    }

    // Check if the leave_type_id exists
    let leave_type_exists = LEAVE_TYPES_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .any(|(id, _)| id == payload.leave_type_id)
    });

    if !leave_type_exists {
        return Err(Message::InvalidPayload(
            "Invalid leave_type_id provided.".to_string(),
        ));
    }

    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    let leave_request = LeaveRequest {
        id,
        employee_id: payload.employee_id,
        leave_type_id: payload.leave_type_id,
        start_date: payload.start_date,
        end_date: payload.end_date,
        status: "pending".to_string(),
        reason: payload.reason,
        created_at: current_time(),
    };
    LEAVE_REQUESTS_STORAGE.with(|storage| storage.borrow_mut().insert(id, leave_request.clone()));
    Ok(leave_request)
}

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

#[ic_cdk::update]
fn approve_leave_request(request_id: u64) -> Result<Message, Message> {
    // Fetch the specific leave request by ID
    let leave_request = LEAVE_REQUESTS_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, request)| request.id == request_id)
            .map(|(_, request)| request.clone())
    });

    if leave_request.is_none() {
        return Err(Message::NotFound("Leave request not found".to_string()));
    }

    let mut request = leave_request.unwrap();

    // Ensure the leave request is not already approved
    if request.status == "approved" {
        return Err(Message::Error(
            "Leave request is already approved.".to_string(),
        ));
    }

    // Fetch the employee associated with the leave request
    let employee = EMPLOYEE_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, employee)| employee.id == request.employee_id)
            .map(|(_, employee)| employee.clone())
    });

    if employee.is_none() {
        return Err(Message::NotFound("Employee not found".to_string()));
    }

    // Fetch the corresponding leave type
    let leave_type = LEAVE_TYPES_STORAGE.with(|storage| {
        storage
            .borrow()
            .get(&request.leave_type_id)
            .map(|lt| lt.clone())
    });

    if leave_type.is_none() {
        return Err(Message::NotFound("Leave type not found".to_string()));
    }

    // Update the leave request status to "approved"
    request.status = "approved".to_string();
    LEAVE_REQUESTS_STORAGE.with(|storage| storage.borrow_mut().insert(request_id, request));

    Ok(Message::Success(
        "Leave request approved and leave balance updated.".to_string(),
    ))
}

#[ic_cdk::update]
fn reject_leave_request(request_id: u64) -> Result<Message, Message> {
    let leave_request = LEAVE_REQUESTS_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, request)| request.id == request_id)
            .map(|(_, request)| request.clone())
    });

    if leave_request.is_none() {
        return Err(Message::NotFound("Leave request not found".to_string()));
    }

    let mut request = leave_request.unwrap();
    request.status = "rejected".to_string();
    LEAVE_REQUESTS_STORAGE.with(|storage| storage.borrow_mut().insert(request_id, request));

    Ok(Message::Success("Leave request rejected.".to_string()))
}

#[ic_cdk::update]
fn create_leave_type(payload: LeaveTypePayload) -> Result<LeaveType, Message> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

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

#[ic_cdk::update]
fn accrue_leave(leave_request_id: u64) -> Result<Message, Message> {
    // Fetch the specific leave request by ID
    let leave_request = LEAVE_REQUESTS_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, request)| request.id == leave_request_id)
            .map(|(_, request)| request.clone())
    });

    if leave_request.is_none() {
        return Err(Message::NotFound("Leave request not found".to_string()));
    }

    let mut leave_request = leave_request.unwrap();

    // Ensure the leave request is approved and not already accrued
    if leave_request.status == "accrued" {
        return Err(Message::Error("Leave request already accrued.".to_string()));
    }

    if leave_request.status != "approved" {
        return Err(Message::Error("Leave request is not approved.".to_string()));
    }

    // Fetch the employee associated with the leave request
    let employee = EMPLOYEE_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, employee)| employee.id == leave_request.employee_id)
            .map(|(_, employee)| employee.clone())
    });

    if employee.is_none() {
        return Err(Message::NotFound("Employee not found".to_string()));
    }

    let mut employee = employee.unwrap();

    // Fetch the corresponding leave type
    let leave_type = LEAVE_TYPES_STORAGE.with(|storage| {
        storage
            .borrow()
            .get(&leave_request.leave_type_id)
            .map(|lt| lt.clone())
    });

    if leave_type.is_none() {
        return Err(Message::NotFound("Leave type not found".to_string()));
    }

    let leave_type = leave_type.unwrap();

    // Fetch the current balance or default to zero if none exists
    let balance = employee
        .leave_balances
        .entry(leave_type.name.clone())
        .or_insert(0);

    // Debug print to check the balance before updating
    ic_cdk::println!("Current balance for {:?}: {}", leave_type.name, balance);

    // Subtract the leave type's quota from the balance
    *balance = balance.saturating_sub(leave_type.quota);

    // Debug print to check the balance after updating
    ic_cdk::println!("Updated balance for {:?}: {}", leave_type.name, *balance);

    // Update the leave request status to "accrued"
    leave_request.status = "accrued".to_string();

    // Update the leave request in the storage
    LEAVE_REQUESTS_STORAGE
        .with(|storage| storage.borrow_mut().insert(leave_request_id, leave_request));

    // Update the employee in the storage
    EMPLOYEE_STORAGE.with(|storage| storage.borrow_mut().insert(employee.id, employee));

    Ok(Message::Success(
        "Leave request accrued successfully and balance updated.".to_string(),
    ))
}

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

    let employee = EMPLOYEE_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, employee)| employee.id == employee_id)
            .map(|(_, employee)| employee.clone())
    });

    if employee.is_none() {
        return Err(Message::NotFound("Employee not found".to_string()));
    }

    let employee = employee.unwrap();

    let mut report = format!(
        "Leave report for {} (Employee ID: {})\n\n",
        employee.name, employee.id
    );

    for request in leave_requests {
        let leave_type = LEAVE_TYPES_STORAGE.with(|storage| {
            storage
                .borrow()
                .iter()
                .find(|(_, leave_type)| leave_type.id == request.leave_type_id)
                .map(|(_, leave_type)| leave_type.clone())
        });

        if leave_type.is_none() {
            return Err(Message::NotFound("Leave type not found".to_string()));
        }

        let leave_type = leave_type.unwrap();
        let leave_balance = employee.leave_balances.get(&leave_type.name).unwrap_or(&0);

        // Correctly calculate the remaining leaves
        let remaining_leaves = *leave_balance;

        report += &format!(
            "Leave Request ID: {}\nLeave Type: {}\nQuota: {}\nRemaining Leaves: {}\nStart Date: {}\nEnd Date: {}\nStatus: {}\nReason: {}\n\n",
            request.id,
            leave_type.name,
            leave_type.quota,
            remaining_leaves,
            request.start_date,
            request.end_date,
            request.status,
            request.reason
        );
    }

    Ok(report)
}

#[ic_cdk::update]
fn cancel_leave_request(request_id: u64) -> Result<Message, Message> {
    let leave_request = LEAVE_REQUESTS_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .find(|(_, request)| request.id == request_id)
            .map(|(_, request)| request.clone())
    });

    if leave_request.is_none() {
        return Err(Message::NotFound("Leave request not found".to_string()));
    }

    let request = leave_request.unwrap();
    if request.status == "approved" {
        return Err(Message::Error(
            "Cannot cancel an approved leave request.".to_string(),
        ));
    }

    LEAVE_REQUESTS_STORAGE.with(|storage| storage.borrow_mut().remove(&request_id));

    Ok(Message::Success("Leave request canceled.".to_string()))
}

fn current_time() -> u64 {
    time()
}

ic_cdk::export_candid!();
