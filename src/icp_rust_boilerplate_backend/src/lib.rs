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
    quota: u32,
    carryover_allowed: bool,
    created_at: u64,
}

// Implementing Storable for Employee, LeaveRequest, and LeaveType
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

// Thread-local storages for managing data
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

// Error-handling enum for messages
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Message {
    Success(String),
    Error(String),
    NotFound(String),
    InvalidPayload(String),
}

// Utility function to get the current time in nanoseconds
fn current_time() -> u64 {
    time()
}

// New Feature: Auto-generating an email if not provided
fn generate_email(name: &str) -> String {
    format!("{}@company.com", name.to_lowercase().replace(' ', "."))
}

// Helper: Get leave type by ID
fn get_leave_type(leave_type_id: u64) -> Result<LeaveType, Message> {
    LEAVE_TYPES_STORAGE.with(|storage| {
        storage
            .borrow()
            .get(&leave_type_id)
            .ok_or(Message::NotFound("Leave type not found".to_string()))
    })
}

// Helper: Increment ID counter
fn increment_id_counter() -> Result<u64, Message> {
    ID_COUNTER.with(|counter: &RefCell<IdCell>| {
        let current_value = {
            let counter_borrow = counter.borrow();  // Immutable borrow to get the value
            *counter_borrow.get()                  // Copy the value out
        };

        {
            let mut counter_borrow = counter.borrow_mut(); // Mutable borrow to set the new value
            counter_borrow
                .set(current_value + 1)
                .expect("Failed to set new ID.");
        }

        Ok(current_value + 1) // Return the incremented value
    })
}




// Create a new employee with error handling
#[ic_cdk::update]
fn create_employee(mut payload: EmployeePayload) -> Result<Employee, Message> {
    if payload.name.is_empty() {
        return Err(Message::InvalidPayload("Employee name is required.".to_string()));
    }

    if payload.email.is_empty() {
        payload.email = generate_email(&payload.name);
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

// Query to fetch all employees
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

// Search employees by name or email
#[ic_cdk::query]
fn search_employee(query: String) -> Result<Vec<Employee>, Message> {
    let employees: Vec<Employee> = EMPLOYEE_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .map(|(_, employee)| employee.clone())
            .filter(|employee| employee.name.contains(&query) || employee.email.contains(&query))
            .collect()
    });

    if employees.is_empty() {
        Err(Message::NotFound("No matching employees found".to_string()))
    } else {
        Ok(employees)
    }
}

// Create a leave request with error handling
#[ic_cdk::update]
fn create_leave_request(payload: LeaveRequestPayload) -> Result<LeaveRequest, Message> {
    if payload.reason.is_empty() {
        return Err(Message::InvalidPayload("Ensure 'reason' is provided.".to_string()));
    }

    // Check if the leave_type_id exists
    let leave_type_exists = LEAVE_TYPES_STORAGE.with(|storage| {
        storage
            .borrow()
            .iter()
            .any(|(id, _)| id == payload.leave_type_id)
    });

    if !leave_type_exists {
        return Err(Message::InvalidPayload("Invalid leave_type_id provided.".to_string()));
    }

    let id = increment_id_counter()?;

    let leave_request = LeaveRequest {
        id,
        employee_id: payload.employee_id,
        leave_type_id: payload.leave_type_id,
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

// Approve a leave request with error handling
#[ic_cdk::update]
fn approve_leave_request(request_id: u64) -> Result<Message, Message> {
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

    if request.status == LeaveStatusEnum::Approved {
        return Err(Message::Error("Leave request is already approved.".to_string()));
    }

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

    let leave_type = get_leave_type(request.leave_type_id)?;

    request.status = LeaveStatusEnum::Approved;
    LEAVE_REQUESTS_STORAGE.with(|storage| storage.borrow_mut().insert(request_id, request));

    Ok(Message::Success("Leave request approved.".to_string()))
}

// Cancel a leave request with error handling
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
    if request.status == LeaveStatusEnum::Approved {
        return Err(Message::Error("Cannot cancel an approved leave request.".to_string()));
    }

    LEAVE_REQUESTS_STORAGE.with(|storage| storage.borrow_mut().remove(&request_id));
    Ok(Message::Success("Leave request canceled.".to_string()))
}

// Generate a leave report for an employee
#[ic_cdk::query]
fn generate_leave_report(employee_id: u64) -> Result<String, Message> {
    let leave_requests = get_employee_leave_requests(employee_id)?;
    let employee = get_employee_by_id(employee_id)?;

    let mut report = format!(
        "Leave report for {} (Employee ID: {})\n\n",
        employee.name, employee.id
    );

    let mut total_days_by_type: HashMap<LeaveName, u32> = HashMap::new();

    for request in leave_requests {
        let leave_type = get_leave_type(request.leave_type_id)?;
        let balance = employee.leave_balances.get(&leave_type.name).unwrap_or(&0);

        *total_days_by_type.entry(leave_type.name).or_insert(0) += leave_type.quota;

        report += &format!(
            "Leave Request ID: {}\nLeave Type: {}\nStart Date: {}\nEnd Date: {}\nStatus: {:?}\nReason: {}\nRemaining Balance: {}\n\n",
            request.id,
            leave_type.name,
            request.start_date,
            request.end_date,
            request.status,
            request.reason,
            balance
        );
    }

    report += "Summary of Leave Taken by Type:\n";
    for (leave_name, total_days) in total_days_by_type {
        report += &format!("{}: {} days\n", leave_name, total_days);
    }

    Ok(report)
}

// Helper function to get employee by ID
fn get_employee_by_id(employee_id: u64) -> Result<Employee, Message> {
    EMPLOYEE_STORAGE.with(|storage| {
        storage
            .borrow()
            .get(&employee_id)
            .ok_or(Message::NotFound("Employee not found".to_string()))
    })
}

// Helper function to get employee leave requests by ID
fn get_employee_leave_requests(employee_id: u64) -> Result<Vec<LeaveRequest>, Message> {
    LEAVE_REQUESTS_STORAGE.with(|storage| {
        let requests: Vec<LeaveRequest> = storage
            .borrow()
            .iter()
            .filter(|(_, request)| request.employee_id == employee_id)
            .map(|(_, request)| request.clone())
            .collect();

        if requests.is_empty() {
            Err(Message::NotFound("No leave requests found".to_string()))
        } else {
            Ok(requests)
        }
    })
}

// Export the Candid interface for the system
ic_cdk::export_candid!();
