
# 🏢 Employee Leave Management System

This project is a decentralized platform built on the Internet Computer for managing employees, leave types, leave requests, and generating leave reports. It allows users to create and manage records for employees, define different leave types, handle leave requests, and track the leave balances of employees.

## ✨ Key Features

### 👥 Employee Management
- **➕ Add Employee**: Allows users to create employee profiles.
- **📋 Get All Employees**: Retrieve a list of all employees in the system.

### 📅 Leave Type Management
- **➕ Create Leave Type**: Allows users to define various leave types (e.g., Annual, Sick, Maternity).
- **📋 Get All Leave Types**: Retrieve a list of all leave types in the system.

### 📝 Leave Request Management
- **➕ Create Leave Request**: Allows users to submit leave requests for employees.
- **✅ Approve Leave Request**: Approve a leave request and adjust the employee's leave balance.
- **❌ Reject Leave Request**: Reject a leave request without affecting the leave balance.
- **🚫 Cancel Leave Request**: Allows users to cancel a pending leave request.
- **🔄 Accrue Leave**: Automatically adjusts the leave balance for approved leave requests.
- **📋 Get All Leave Requests**: Retrieve a list of all leave requests in the system.

### 📊 Leave Report Management
- **📄 Generate Leave Report**: Generate a detailed report of all leave requests for a specific employee.

## ⚠️ Error Handling

- **🔍 Not Found**: Returns an error if a requested employee, leave type, or leave request is not found.
- **❗ Invalid Payload**: Returns an error if the provided data is incomplete or invalid.
- **🚫 Unauthorized Action**: Returns an error if an action is not permitted, such as trying to cancel an already approved leave request.

## 🛠️ Sample Payloads

### 🧑‍💼 EmployeePayload

```json
{
  "name": "John Doe",
  "email": "john.doe@example.com"
}
```

### 📅 LeaveTypePayload

```json
{
  "name": "Annual",
  "quota": 20,
  "carryover_allowed": true
}
```

### 📝 LeaveRequestPayload

```json
{
  "employee_id": 1,
  "leave_type_id": 1,
  "start_date": 1638316800,
  "end_date": 1638403200,
  "reason": "Family vacation"
}
```

## 💻 Usage Examples using `curl`

### ➕ Add an Employee

```bash
dfx canister call employee_leave_management_system create_employee '(record { name = "John Doe"; email = "john.doe@example.com" })'
```

### ➕ Create a Leave Type

```bash
dfx canister call employee_leave_management_system create_leave_type '(record { name = variant { Annual }; quota = 20; carryover_allowed = true })'
```

### 📝 Submit a Leave Request

```bash
dfx canister call employee_leave_management_system create_leave_request '(record { employee_id = 1; leave_type_id = 1; start_date = 1638316800; end_date = 1638403200; reason = "Family vacation" })'
```

### ✅ Approve a Leave Request

```bash
dfx canister call employee_leave_management_system approve_leave_request '(1)'
```

### 📄 Generate a Leave Report

```bash
dfx canister call employee_leave_management_system generate_leave_report '(1)'
```

## 🛣️ User Flow

### 1. 👥 Employee Management
- **Admin** logs into the system.
- **Admin** adds a new employee by filling in the employee's name and email.
- **Admin** can view a list of all employees in the system.

### 2. 📅 Leave Type Management
- **Admin** creates different leave types (e.g., Annual, Sick) by specifying the name, quota, and whether carryover is allowed.
- **Admin** can view a list of all leave types that have been created.

### 3. 📝 Leave Request Management
- **Employee** logs into the system.
- **Employee** submits a leave request by selecting the leave type, start and end dates, and providing a reason.
- **Admin** reviews the leave request.
  - If approved, the leave balance for the employee is adjusted.
  - If rejected, the leave request is marked as such without affecting the leave balance.
- **Employee** can cancel a leave request if it is still pending.
- **System** automatically accrues leave balances based on approved leave requests.

### 4. 📊 Leave Report Management
- **Admin** generates a leave report for any employee, showing all their leave requests, including dates, types, and approval status.

### 5. ⚠️ Error Handling
- If an **Employee** or **Admin** tries to access a non-existent employee, leave type, or leave request, the system returns a "Not Found" error.
- If an **Employee** or **Admin** provides incomplete or invalid data, the system returns an "Invalid Payload" error.
- If an **Admin** attempts to perform an unauthorized action, like canceling an already approved leave request, the system returns an "Unauthorized Action" error.

## 📋 Requirements

- rustc 1.64 or higher

```bash
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
$ source "$HOME/.cargo/env"
```

- rust wasm32-unknown-unknown target

```bash
$ rustup target add wasm32-unknown-unknown
```

- candid-extractor

```bash
$ cargo install candid-extractor
```

- install `dfx`

```bash
$ DFX_VERSION=0.15.0 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
$ echo 'export PATH="$PATH:$HOME/bin"' >> "$HOME/.bashrc"
$ source ~/.bashrc
$ dfx start --background
```

If you want to start working on your project right away, you might want to try the following commands:

```bash
$ cd icp_rust_boilerplate/
$ dfx help
$ dfx canister --help
```

## 🔄 Update dependencies

Update the `dependencies` block in `/src/{canister_name}/Cargo.toml`:

```
[dependencies]
candid = "0.9.9"
ic-cdk = "0.11.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
ic-stable-structures = { git = "https://github.com/lwshang/stable-structures.git", branch = "lwshang/update_cdk"}
```

## 🔧 did autogenerate

Add this script to the root directory of the project:

```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh
```

Update line 16 with the name of your canister:

```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh#L16
```

After this, run this script to generate Candid.
Important note!

You should run this script each time you modify/add/remove exported functions of the canister.
Otherwise, you'll have to modify the candid file manually.

Also, you can add a package.json with this content:

```
{
    "scripts": {
        "generate": "./did.sh && dfx generate",
        "gen-deploy": "./did.sh && dfx generate && dfx deploy -y"
      }
}
```

and use commands `npm run generate` to generate candid or `npm run gen-deploy` to generate candid and to deploy a canister.

## 🧪 Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
$ dfx start --background

# Deploys your canisters to the replica and generates your candid interface
$ dfx deploy
```