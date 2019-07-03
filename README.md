# SaasRRBAC - RBAC implementation for SAAS based applications, written in Rust


## Overview

SaasRRBAC provides APIs to add support for role and claim based security. It supports both static and instance based security.


## Features:
- Multi-tenancy support for multiple organizations and users within those organizations
- Organizations can have multiple users (principals) and groups
- Role based security
- Roles can be inherited
- Roles can be associated with principals or groups of organizations
- Claim based security, where claims define permittable actions and can be associated directly with principals or
  via roles that are associated to principals.
- Instance/Constraints based security using dynamic context.
- Roles and claims can be assigned within a range of time period.
- Restrict access based on geo-fencing
- Resource accountings, where you can limit and track usage by users and use them as part of instance based security.

## Requirements:
- Rust

## Version
- 0.1

## License
- MIT

## Design:


### Realm

The realm defines domain of security, e.g. you may have have different security policies for different applications that can be applied by creating realm for each application.

### Principal

A principal represents an identity and can be represented by user or an application.

### Role

A role represents job title or function. A principal belongs to one or more roles. One of key feature of SaasRRBAC is that roles support inheritance where a role can have one or more roles. Roles can be assigned for a predefined duration of time to principals.

### Claim

A claim defines permission and consists of three parts: operation, resource and condition, where operation is a "verb" that describes action and resource represents "object" that is acted upon, and condition is an optional component that describes dynamic condition that must be checked. The claims can be assigned to roles or principal.
The condition contains a Javascript based logical expressions and provides access to runtime request parameters. Claim can be assigned for a duration of time so that they are not permanent.


## System Layers

SaasRRBAC consists of following layers
### Business Domain Layer
<img src="https://raw.githubusercontent.com/bhatti/saas_rbac/master/images/uml.png">

This layer defines core classes that are part of the RBAC based security realm such as:

 * Realm – - The realm allows you to support multiple applications or security realms.
 * Organization – The organization represents the organization that customer belongs to, each customer
    may belong to different organizations.
 * Principal – The principal represents an identity of users that belong to customer organizations. Note,
    this object represents employees/users of customers and not users of hosting provider, though they can
    be modeled in similar fashion.
 * LicensePolicy – The license policy represents overall access for the customers and it's mapped to claims.
 * Role – A role represents job title or function.
 * Resource - The resource represents object that needs to be protected such as APIs, files, data, reports, etc.
 * Claim – A claim is tied with resources and defines operation and constraints that need to be inforced. It may
   define dynamic properties for for dynamic or instance based security.
 * SecurityManager – Checks access permission for principals.

### Repository Layer

This layer is responsible for accessing or storing above objects in the database. SaasRRBAC uses Sqlite by default but it can be easily mapped to other databases. Following are list of repositories supported by SaasRRBAC:

	* PersistenceManager – provides high level methods to persist or query domain objects for security
	* RealmRepository – provides database access for Realms.
	* ClaimRepository – provides database access for Claims.
	* PrincipalRepository – provides database access for Principals.
	* RoleRepository – provides database access for Roles.
	* GroupRepository – provides database access for Groups.
	* LicensePolicyRepository – provides database access for license-policy

### Security Layer

This layer defines SecurityManager for validating authorization policies.

### Evaluation Layer

This layer proivdes evaluation engine for supporting instance based security.

### REST API Service Layer

This layer defines REST services such as:

	* RealmService – this service provides REST APIs for accessing Realms.
	* OrganizationService – this service provides REST APIs for accessing Organizations
	* PrincipalService – this service provides REST APIs for accessing Principals.
    * LicensePolicyService – this service provides REST APIs for accessing license policies.
	* RoleService – this service provides REST APIs for accessing Roles.
	* ResourceService – this service provides REST APIs for accessing resources, instances, and limits.
	* ClaimService – this service provides REST APIs for accessing Claims.
	* SecurityService – this service provides REST APIs for authorizing claims.


### Caching Layer

This layer provides caching security claims to improve performance.

### Setup
 - Install rust
```
rustup override set nightly
rustup update && cargo update
```
 - Run migrations
```
cargo install diesel_cli --no-default-features --features sqlite
cargo install cargo-tree
echo DATABASE_URL=db.qlite > .env
diesel setup --database-url db.sqlite
diesel migration run
diesel print-schema > src/plexrbac/persistence/schema.rs
```

Note: By default, SaasRRBAC works with sqlite but you can update Cargo.toml and above command to use postgres or mysql.
Note: You can re-apply migrations using diesel migration redo (See https://sqliteonline.com/)

 - Run Tests
```
cargo test -- --test-threads=1
```

 - Run Test Coverage
```
brew install cmake jq
cargo kcov --print-install-kcov-sh | sh
cd kcov-v36
cmake -G Xcode
xcodebuild -configuration Release

cargo install cargo-kcov
```

 - Docs
```
rustc doc.rs --crate-type lib
rustdoc --test --extern doc="libdoc.rlib" doc.rs
```


## Use Cases

### Banking

Let’s start with a banking example where a bank-object can be account, general-ledger-report or ledger-posting-rules and account is further grouped into customer account or loan account. Further, Let’s assume there are five roles: Teller, Customer-Service-Representative (CSR), Account, AccountingManager and LoanOfficer, where

 - A teller can modify customer deposit accounts — but only if customer and teller live in same region
 - A customer service representative can create or delete customer deposit accounts — but only if customer and teller live in same region
 - An accountant can create general ledger reports — but only if year is h2. current year
 - An accounting manager can modify ledger-posting rules — but only if year is h2. current year
 - A loan officer can create and modify loan accounts – but only if account balance is < 10000

Bootstrapping dependent data

```rust
let ctx = SecurityContext::new("0".into(), "0".into());
let factory = RepositoryFactory::new();
let mgr = factory.new_persistence_manager();
```
Creating security realm

```rust
let realm = mgr.new_realm_with(&ctx, "banking")?;
```

Creating organization
```rust
let org = mgr.new_org_with(&ctx, "bank-of-flakes")?;
```

Creating Users
```rust
let tom = mgr.new_principal_with(&ctx, &org, "tom")?;
let cassy = mgr.new_principal_with(&ctx, &org, "cassy")?;
let ali = mgr.new_principal_with(&ctx, &org, "ali")?;
let mike = mgr.new_principal_with(&ctx, &org, "mike")?;
let larry = mgr.new_principal_with(&ctx, &org, "larry")?;
```

Creating Roles
```rust
let employee = mgr.new_role_with(&ctx, &realm, &org, "Employee")?;
let teller = mgr.new_role_with_parent(&ctx, &realm, &org, &employee, "Teller")?;
let csr = mgr.new_role_with_parent(&ctx, &realm, &org, &teller, "CSR")?;
let accountant = mgr.new_role_with_parent(&ctx, &realm, &org, &employee, "Accountant")?;
let accountant_manager = mgr.new_role_with_parent(&ctx, &realm, &org, &accountant, "AccountingManager")?;
let loan_officer = mgr.new_role_with_parent(&ctx, &realm, &org, &accountant_manager, "LoanOfficer")?;
```

Creating Resources
```rust
let deposit_account = mgr.new_resource_with(&ctx, &realm, "DepositAccount")?;
let loan_account = mgr.new_resource_with(&ctx, &realm, "LoanAccount")?;
let general_ledger = mgr.new_resource_with(&ctx, &realm, "GeneralLedger")?;
let posting_rules = mgr.new_resource_with(&ctx, &realm, "GeneralLedgerPostingRules")?;
```

Creating claims for resources
```rust
let cd_deposit = mgr.new_claim_with(&ctx, &realm, &deposit_account, "(CREATE|DELETE)")?;
let ru_deposit = mgr.new_claim_with(&ctx, &realm, &deposit_account, "(READ|UPDATE)")?;

let cd_loan = mgr.new_claim_with(&ctx, &realm, &loan_account, "(CREATE|DELETE)")?;
let ru_loan = mgr.new_claim_with(&ctx, &realm, &loan_account, "(READ|UPDATE)")?;

let rd_ledger = mgr.new_claim_with(&ctx, &realm, &general_ledger, "(READ|CREATE)")?;
let r_glpr = mgr.new_claim_with(&ctx, &realm, &general_ledger, "(READ)")?;

let cud_glpr = mgr.new_claim_with(&ctx, &realm, &posting_rules, "(CREATE|UPDATE|DELETE)")?;
```

Mapping Principals and Claims to Roles
```rust
mgr.map_principal_to_role(&ctx, &tom, &teller);
mgr.map_principal_to_role(&ctx, &cassy, &csr);
mgr.map_principal_to_role(&ctx, &ali, &accountant);
mgr.map_principal_to_role(&ctx, &mike, &accountant_manager);
mgr.map_principal_to_role(&ctx, &larry, &loan_officer);
```

Map claims to roles as follows:
```rust
mgr.map_role_to_claim(&ctx, &teller, &ru_deposit, "U.S.", r#"employeeRegion == "Midwest""#);
mgr.map_role_to_claim(&ctx, &csr, &cd_deposit, "U.S.", r#"employeeRegion == "Midwest""#);
mgr.map_role_to_claim(&ctx, &accountant, &rd_ledger, "U.S.", r#"employeeRegion == "Midwest" && ledgerYear == current_year()"#);
mgr.map_role_to_claim(&ctx, &accountant, &ru_loan, "U.S.", r#"employeeRegion == "Midwest" && accountBlance < 10000"#);
mgr.map_role_to_claim(&ctx, &accountant_manager, &cd_loan, "U.S.", r#"employeeRegion == "Midwest" && accountBlance < 10000"#);
mgr.map_role_to_claim(&ctx, &accountant_manager, &r_glpr, "U.S.", r#"employeeRegion == "Midwest" && ledgerYear == current_year()"#);
mgr.map_role_to_claim(&ctx, &loan_officer, &cud_glpr, "U.S.", r#"employeeRegion == "Midwest" && ledgerYear == current_year()"#);
```

Checking permissions
```rust
let security_mgr = SecurityManager::new(mgr);
```

Tom, the teller should be able to READ DepositAccount with scope U.S when employeeRegion == Midwest
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::READ, "DepositAccount", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
assert_eq!(PermissionResult::Allow, security_mgr.check(&req)?);
```

Tom, the teller should not be able to READ DepositAccount with scope U.S when employeeRegion == Northeast
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::READ, "DepositAccount", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Northeast".to_string()));
assert!(security_mgr.check(&req).is_err());
```

Tom, the teller should not be able to DELETE DepositAccount with scope U.S when employeeRegion == Midwest
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::DELETE, "DepositAccount", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
assert!(security_mgr.check(&req).is_err());
```

Cassy, the CSR should be able to DELETE DepositAccount with scope U.S when employeeRegion == Midwest
```rust
let mgr = factory.new_persistence_manager();
let mut req = PermissionRequest::new(realm.id.as_str(), cassy.id.as_str(), ActionType::DELETE, "DepositAccount", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
assert_eq!(PermissionResult::Allow, security_mgr.check(&req)?);
```

Cassy, the CSR should be able to DELETE DepositAccount with scope U.K when employeeRegion == Midwest
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), cassy.id.as_str(), ActionType::DELETE, "DepositAccount", "U.K.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
assert!(security_mgr.check(&req).is_err());
```

Ali, the Accountant should be able to READ GeneralLedger with scope U.S when employeeRegion == Midwest AND ledgerYear == current_year()
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::READ, "GeneralLedger", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
assert_eq!(PermissionResult::Allow, security_mgr.check(&req)?);
```

Ali, the Accountant should not be able to READ GeneralLedger with scope U.S when employeeRegion == Midwest AND ledgerYear is in past
```rust
req.context.add("ledgerYear", ValueWrapper::Int(2000));
assert!(security_mgr.check(&req).is_err());
```

Ali, the Accountant should not be able to DELETE GeneralLedger with scope U.S when employeeRegion == Midwest AND ledgerYear == current_year()
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::DELETE, "GeneralLedger", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
assert!(security_mgr.check(&req).is_err());
```

Mike, the Accountant Manager should be able to DELETE GeneralLedger with scope U.S when employeeRegion == Midwest AND ledgerYear == current_year()
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::CREATE, "GeneralLedger", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
assert_eq!(PermissionResult::Allow, security_mgr.check(&req)?);
```


Mike, the Accountant Manager should not be able to post posting-rules of general-ledger with scope U.S when employeeRegion == Midwest AND ledgerYear == current_year()
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::CREATE, "GeneralLedgerPostingRules", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
req.context.add("accountBlance", ValueWrapper::Int(500));
assert!(security_mgr.check(&req).is_err());
```

Larry, the Loan Officer should be able to post posting-rules of general-ledger with scope U.S when employeeRegion == Midwest AND ledgerYear == current_year()
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), larry.id.as_str(), ActionType::CREATE, "GeneralLedgerPostingRules", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
req.context.add("accountBlance", ValueWrapper::Int(500));
assert_eq!(PermissionResult::Allow, security_mgr.check(&req)?);
```

### Expense Report

Initialize context and repository
```rust
let ctx = SecurityContext::new("0".into(), "0".into());
let factory = RepositoryFactory::new();
let mgr = factory.new_persistence_manager();
mgr.clear();
```

Creating security realm
```rust
let realm = mgr.new_realm_with(&ctx, "expense")?;
```

Creating organization
```rust
let org = mgr.new_org_with(&ctx, "box-air")?;
```

Creating Groups
```rust
let group_employee = mgr.new_group_with(&ctx, &org, "Employee")?;
let group_manager = mgr.new_group_with_parent(&ctx, &org, &group_employee, "Manager")?;
```

Creating Users
```rust
let tom = mgr.new_principal_with(&ctx, &org, "tom")?;
let mike = mgr.new_principal_with(&ctx, &org, "mike")?;
```

Mapping users to groups
```rust
mgr.map_principal_to_group(&ctx, &tom, &group_employee);
mgr.map_principal_to_group(&ctx, &mike, &group_employee);
mgr.map_principal_to_group(&ctx, &mike, &group_manager);
```

Creating Roles
```rust
let employee = mgr.new_role_with(&ctx, &realm, &org, "Employee")?;
let manager = mgr.new_role_with_parent(&ctx, &realm, &org, &employee, "Manager")?;
```

Creating Resources
```rust
let expense_report = mgr.new_resource_with(&ctx, &realm, "ExpenseReport")?;
```

Creating claims for resources
```rust
let submit_report = mgr.new_claim_with(&ctx, &realm, &expense_report, "(SUBMIT|VIEW)")?;
let approve_report = mgr.new_claim_with(&ctx, &realm, &expense_report, "APPROVE")?;
```

Mapping Principals and Claims to Roles
```rust
mgr.map_group_to_role(&ctx, &group_employee, &employee, "");
mgr.map_group_to_role(&ctx, &group_manager, &manager, "");
```

Map claims to roles as follows:
```rust
mgr.map_role_to_claim(&ctx, &employee, &submit_report, "U.S.", r#"amount < 10000"#);
mgr.map_role_to_claim(&ctx, &manager, &approve_report, "U.S.", r#"amount < 10000"#);
```

Checking Permissions
```rust
let security_mgr = SecurityManager::new(mgr);
```

Tom should be able to submit report
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::SUBMIT, "ExpenseReport", "U.S.");
req.context.add("amount", ValueWrapper::Int(1000));
assert_eq!(PermissionResult::Allow, security_mgr.check(&req)?);
```

Tom should not be able to approve report
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::APPROVE, "ExpenseReport", "U.S.");
req.context.add("amount", ValueWrapper::Int(1000));
assert!(security_mgr.check(&req).is_err());
```

Mike should be able to approve report
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::APPROVE, "ExpenseReport", "U.S.");
req.context.add("amount", ValueWrapper::Int(1000));
assert_eq!(PermissionResult::Allow, security_mgr.check(&req)?);
```

### Feature flag with Geo-Fencing
Initialize context and repository
```rust
let ctx = SecurityContext::new("0".into(), "0".into());
let factory = RepositoryFactory::new();
let mgr = factory.new_persistence_manager();
mgr.clear();
```

Creating security realm
```rust
let realm = mgr.new_realm_with(&ctx, "ada")?;
```

Creating organization
```rust
let org = mgr.new_org_with(&ctx, "ada")?;
```

Creating Users
```rust
let tom = mgr.new_principal_with(&ctx, &org, "tom")?;
let mike = mgr.new_principal_with(&ctx, &org, "mike")?;
```

Creating Roles
```rust
let customer = mgr.new_role_with(&ctx, &realm, &org, "Customer")?;
let beta_customer = mgr.new_role_with_parent(&ctx, &realm, &org, &customer, "BetaCustomer")?;
```

Creating Resources
```rust
let feature = mgr.new_resource_with(&ctx, &realm, "Feature")?;
```

Creating claims for resources
```rust
let view = mgr.new_claim_with(&ctx, &realm, &feature, "VIEW")?;
```

Mapping Principals and Claims to Roles
```rust
mgr.map_principal_to_role(&ctx, &tom, &customer);
mgr.map_principal_to_role(&ctx, &mike, &beta_customer);
```

Map claims to roles as follows:
```rust
mgr.map_role_to_claim(&ctx, &customer, &view, "UI::Flag::BasicReport", r#"geo_distance_km(customer_lat, customer_lon, 47.620422, -122.349358) < 100"#);
mgr.map_role_to_claim(&ctx, &beta_customer, &view, "UI::Flag::AdvancedReport", r#"geo_distance_km(customer_lat, customer_lon, 47.620422, -122.349358) < 200"#);
```

Tom should be able to view basic report if he lives close to Seattle
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::BasicReport");
req.context.add("customer_lat", ValueWrapper::Float(46.879967));
req.context.add("customer_lon", ValueWrapper::Float(-121.726906));
assert_eq!(PermissionResponse::Allow, security_mgr.check(&req)?);
```

Tom should not be able to view basic report if he lives far from Seattle
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::BasicReport");
req.context.add("customer_lat", ValueWrapper::Float(37.3230));
req.context.add("customer_lon", ValueWrapper::Float(-122.0322));
assert!(security_mgr.check(&req).is_err());
```

Tom should not be able to view advanced report
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
req.context.add("customer_lat", ValueWrapper::Float(46.879967));
req.context.add("customer_lon", ValueWrapper::Float(-121.726906));
assert!(security_mgr.check(&req).is_err());
```

Mike should be able to view advanced report
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
req.context.add("customer_lat", ValueWrapper::Float(46.879967));
req.context.add("customer_lon", ValueWrapper::Float(-121.726906));
assert_eq!(PermissionResponse::Allow, security_mgr.check(&req)?);
```

Mike should not be able to view advanced report if he lives far from Seattle
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
req.context.add("customer_lat", ValueWrapper::Float(37.3230));
req.context.add("customer_lon", ValueWrapper::Float(-122.0322));
assert!(security_mgr.check(&req).is_err());
```

### Using License Policy to restrict access to different level of features

// Creating organization
```rust
let freemium_org = mgr.new_org_with(&ctx, "Freeloader")?;
let paid_org = mgr.new_org_with(&ctx, "Moneymaker")?;
```

Create license policies
```rust
let freemium_policy = mgr.new_license_policy(&ctx, &freemium_org)?;
let paid_policy = mgr.new_license_policy(&ctx, &paid_org)?;
```

Creating Users
```rust
let freemium_frank = mgr.new_principal_with(&ctx, &freemium_org, "frank")?;
let money_matt = mgr.new_principal_with(&ctx, &paid_org, "matt")?;
```

Creating Roles
```rust
let customer = mgr.new_role_with(&ctx, &realm, &freemium_org, "Customer")?;
let paid_customer = mgr.new_role_with(&ctx, &realm, &paid_org, "PaidCustomer")?;
```

Creating Resources
```rust
let feature = mgr.new_resource_with(&ctx, &realm, "Feature")?;
```

Creating claims for resources
```rust
let view = mgr.new_claim_with(&ctx, &realm, &feature, "VIEW")?;
```

Mapping Principals and Claims to Roles
```rust
mgr.map_principal_to_role(&ctx, &freemium_frank, &customer);
mgr.map_principal_to_role(&ctx, &money_matt, &customer);
mgr.map_principal_to_role(&ctx, &money_matt, &paid_customer);
```

Map claims to policies as follows:
```rust
mgr.map_license_policy_to_claim(&ctx, &freemium_policy, &view, "UI::Flag::BasicReport", "");
mgr.map_license_policy_to_claim(&ctx, &paid_policy, &view, "UI::Flag::AdvancedReport", "");
```

Map claims to roles as follows:
```rust
mgr.map_role_to_claim(&ctx, &customer, &view, "UI::Flag::BasicReport", "");
mgr.map_role_to_claim(&ctx, &paid_customer, &view, "UI::Flag::AdvancedReport", "");
```

Frank should be able to view basic report
```rust
let req = PermissionRequest::new(realm.id.as_str(), freemium_frank.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::BasicReport");
assert_eq!(PermissionResponse::Allow, security_mgr.check(&req)?);
```

Frank should not be able to view advanced report
```rust
let req = PermissionRequest::new(realm.id.as_str(), freemium_frank.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
assert!(security_mgr.check(&req).is_err());
```

Matt should be able to view advanced report
```rust
let req = PermissionRequest::new(realm.id.as_str(), money_matt.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
assert_eq!(PermissionResponse::Allow, security_mgr.check(&req)?);
```
### Using Multiple teams for roles
Create license policies
```rust
let policy = mgr.new_license_policy(&ctx, &org)?;
```

Creating Users
```rust
let dave = mgr.new_principal_with(&ctx, &org, "dave")?;
let qari = mgr.new_principal_with(&ctx, &org, "qari")?;
let ali = mgr.new_principal_with(&ctx, &org, "ali")?;
```

Creating Roles
```rust
let developer = mgr.new_role_with(&ctx, &realm, &org, "Developer")?;
let qa = mgr.new_role_with(&ctx, &realm, &org, "QA")?;
let admin = mgr.new_role_with_parent(&ctx, &realm, &org, &developer, "Admin")?;
```

Creating Resources
```rust
let app = mgr.new_resource_with(&ctx, &realm, "App")?;
```

Creating claims for resources
```rust
let submit_view = mgr.new_claim_with(&ctx, &realm, &app, "(SUBMIT|VIEW)")?;
let view = mgr.new_claim_with(&ctx, &realm, &app, "VIEW")?;
let create_delete = mgr.new_claim_with(&ctx, &realm, &app, "(CREATE|DELETE)")?;
```

Mapping Principals and Claims to Roles
```rust
mgr.map_principal_to_role(&ctx, &dave, &developer);
mgr.map_principal_to_role(&ctx, &qari, &qa);
mgr.map_principal_to_role(&ctx, &ali, &admin);
```

Map claims to policies as follows:
```rust
mgr.map_license_policy_to_claim(&ctx, &policy, &submit_view, "com.xyz.app", "appSize < 1000");
mgr.map_license_policy_to_claim(&ctx, &policy, &view, "com.xyz.app", "appSize < 1000");
mgr.map_license_policy_to_claim(&ctx, &policy, &create_delete, "com.xyz.app", "");
```

Map claims to roles as follows:
```rust
mgr.map_role_to_claim(&ctx, &developer, &submit_view, "com.xyz.app", "appSize < 1000");
mgr.map_role_to_claim(&ctx, &qa, &view, "com.xyz.app", "appSize < 1000");
mgr.map_role_to_claim(&ctx, &admin, &create_delete, "com.xyz.app", "");
```

Dave should be able to submit app
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), dave.id.as_str(), ActionType::SUBMIT, "App", "com.xyz.app");
req.context.add("appSize", ValueWrapper::Int(500));
assert_eq!(PermissionResponse::Allow, security_mgr.check(&req)?);
```

Qari should be able to view app
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), qari.id.as_str(), ActionType::VIEW, "App", "com.xyz.app");
req.context.add("appSize", ValueWrapper::Int(500));
assert_eq!(PermissionResponse::Allow, security_mgr.check(&req)?);
```

Qari should not be able to create app
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), qari.id.as_str(), ActionType::CREATE, "App", "com.xyz.app");
req.context.add("appSize", ValueWrapper::Int(500));
assert!(security_mgr.check(&req).is_err());
```

Ali should be able to create app
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::CREATE, "App", "com.xyz.app");
req.context.add("appSize", ValueWrapper::Int(500));
assert_eq!(PermissionResponse::Allow, security_mgr.check(&req)?);
```

Ali should be able to submit app
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::SUBMIT, "App", "com.xyz.app");
req.context.add("appSize", ValueWrapper::Int(500));
assert_eq!(PermissionResponse::Allow, security_mgr.check(&req)?);
```

Ali should not be able to submit app with large app
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::SUBMIT, "App", "com.xyz.app");
req.context.add("appSize", ValueWrapper::Int(5000));
assert!(security_mgr.check(&req).is_err());
```

## Contact
Please send questions or suggestions to bhatti AT plexobject.com.



## References
 * https://csrc.nist.gov/csrc/media/projects/role-based-access-control/documents/rbac-std-draft.pdf
 * https://www.cs.uic.edu/~ifc/webpapers/CollCom-ready-3.pdf
 * https://people.csail.mit.edu/lkagal/papers/rowlbac.pdf
 * http://www.cis.syr.edu/~wedu/Teaching/cis643/LectureNotes_New/RBAC.pdf
 * http://www.cis.syr.edu/~wedu/seed/Labs/RBAC_Linux/RBAC_Linux.pdf
 * http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.86.9736&rep=rep1&type=pdf
 * http://csrc.nist.gov/groups/SNS/rbac/documents/design_implementation/Intro_role_based_access.htm
 * http://hissa.nist.gov/rbac/poole/ir5820/ir5820s31.htm
 * http://www.coresecuritypatterns.com/patterns.htm
 * http://cwiki.apache.org/confluence/display/SHIRO/Index
 * http://www.secs.oakland.edu/~kim2/papers/FASE04.pdf 
 * http://www.mecs-press.net/ijmecs/ijmecs-v3-n5/IJMECS-V3-N5-7.pdf 
 * http://csrc.nist.gov/groups/SNS/rbac/documents/design_implementation/pp-rbac-fin.pdf
 * https://www.ecs.csus.edu/wcm/csc/pdfs/technical%20reports/role%20based%20access%20control.pdf
 * https://www.researchgate.net/publication/220739294_Role-Based_Access_Control_and_the_Access_Control_Matrix
 * http://shiro.apache.org/
 * https://docs.sensu.io/sensu-enterprise-dashboard/2.15/rbac/rbac-for-oidc/
 * https://medium.com/@jessgreb01/kubernetes-authn-authz-with-google-oidc-and-rbac-74509ca8267e
 * https://docs.aws.amazon.com/cognito/latest/developerguide/role-based-access-control.html
 * https://aws.amazon.com/iam/
 * https://github.com/OWASP/CheatSheetSeries
 * https://blogs.iuvotech.com/rbac-rule-based-vs.-role-based-access-control
 * https://pdfs.semanticscholar.org/72cf/bd0890b64b011cf840acb53d6e1a6337b2b1.pdf
 * https://csrc.nist.gov/CSRC/media/Projects/Role-Based-Access-Control/documents/sandhu96.pdf
 * https://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.336.3000&rep=rep1&type=pdf
 * https://www.cs.purdue.edu/homes/ninghui/papers/rbac_analysis_tissec.pdf
 * https://csrc.nist.gov/projects/role-based-access-control/faqs
 * https://www.researchgate.net/publication/221204798_A_secure_constraint-aware_role-based_access_control_interoperation_framework
 * https://www.codeproject.com/Articles/875547/Custom-Roles-Based-Access-Control-RBAC-in-ASP-NET
 * https://www.cs.purdue.edu/homes/ninghui/papers/aboutRBACStandard.pdf
 * https://docs.oracle.com/cd/E18752_01/html/816-4557/rbac-1.html
 * https://docs.oracle.com/cd/E27515_01/common/tutorials/general_rbac_ldap.html
 * https://ldapcon.org/2011/downloads/gietzwidmer-slides.pdf
 * http://meritsystems.com/an-architectural-approach-to-rbac-policy-management-framework-in-a-soa-driven-enterprise/
 * http://docs.oasis-open.org/xacml/2.0/access_control-xacml-2.0-core-spec-os.pdf
 * https://github.com/eyedia/aarbac
 * https://csrc.nist.gov/projects/role-based-access-control
 * https://www.nist.gov/topics/identity-access-management
 * https://olegkrivtsov.github.io/using-zend-framework-3-book/html/en/Role_Based_Access_Control/Introduction_to_RBAC.html
 * https://developer.okta.com/books/api-security/authz/role-based/
 * https://blog.dereferenced.org/what-is-ocap-and-why-should-i-care
 * https://developer.okta.com/blog/2017/10/13/okta-groups-spring-security
 * https://auth0.com/docs/authorization/concepts/rbac
