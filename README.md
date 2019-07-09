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
- Resource accountings, where you can enforce quota-limit and track usage by users and use them as part of instance based security.

## Requirements:
- Rust

## Version
- 0.1

## License
- MIT

## Design:


### Realm

The realm defines domain of security, e.g. you may have have different security policies for different applications that can be applied by creating realm for each application.

### Organization

An organization represents customer who can have one or more principals (users) and groups.

### Group

A group represents segregation of responsibility within the organization and can be associated with one or more principals (users).

### Principal

A principal represents an identity and tied with the organization.

### Role

A role represents job title or function. A principal belongs to one or more roles. One of key feature of SaasRRBAC is that roles support inheritance where a role can have one or more roles. Roles can be assigned for a predefined duration of time to principals.

### Resource

A resource represents the entity that needs to be protected such as APIs, Data, Feature-set, reports, jobs, projects, etc.

### Claim

A claim defines permission and consists of three parts: operation, resource-id and constraints, where operation is a "verb" that describes action and resource-id represents id of the resource that is acted upon, and constraints is an optional component that describes dynamic condition that must be checked. The claims can be assigned to roles or principal.
The constraints contains a logical expressions and provides access to runtime request parameters. Claim can be assigned for a duration of time so that they are not permanent.

### License Policy

The license policy represents a set of claims that an organization can access based on pricing or license model.

***Note***: The resources and claims are defined by the Saas provider and then sign up process defines organization and license-policy. The organization then creates principals/roles and associates claims with roles/principals. All claims set by the organization would be subset of license policy and time bound within the range of license policy.

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
  * Group – A group represents departments/groups within the organizaiton.
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
  * ResourceService – this service provides REST APIs for accessing resources, instances, and quota-limits.
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
cargo install cargo-kcov
brew install cmake jq
cargo kcov --print-install-kcov-sh | sh
cd kcov-v36
cmake -G Xcode
xcodebuild -configuration Release
cd back-to-rbac-folder
cargo kcov
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


Initialize context and repository
```rust
let ctx = SecurityContext::new("0".into(), "0".into());
let cf = DefaultConnectionFactory::new();
let locator = RepositoryFactory::new(&cf);
let pm = locator.new_persistence_manager();
```

Creating security realm

```rust
let realm = pm.new_realm_with(&ctx, "banking")?;
```

Creating organization
```rust
let org = pm.new_org_with(&ctx, "bank-of-flakes")?;
```

Creating Users
```rust
let tom = pm.new_principal_with(&ctx, &org, "tom")?;
let cassy = pm.new_principal_with(&ctx, &org, "cassy")?;
let ali = pm.new_principal_with(&ctx, &org, "ali")?;
let mike = pm.new_principal_with(&ctx, &org, "mike")?;
let larry = pm.new_principal_with(&ctx, &org, "larry")?;
```

Creating Roles
```rust
let employee = pm.new_role_with(&ctx, &realm, &org, "Employee")?;
let teller = pm.new_role_with_parent(&ctx, &realm, &org, &employee, "Teller")?;
let csr = pm.new_role_with_parent(&ctx, &realm, &org, &teller, "CSR")?;
let accountant = pm.new_role_with_parent(&ctx, &realm, &org, &employee, "Accountant")?;
let accountant_manager = pm.new_role_with_parent(&ctx, &realm, &org, &accountant, "AccountingManager")?;
let loan_officer = pm.new_role_with_parent(&ctx, &realm, &org, &accountant_manager, "LoanOfficer")?;
```

Creating Resources
```rust
let deposit_account = pm.new_resource_with(&ctx, &realm, "DepositAccount")?;
let loan_account = pm.new_resource_with(&ctx, &realm, "LoanAccount")?;
let general_ledger = pm.new_resource_with(&ctx, &realm, "GeneralLedger")?;
let posting_rules = pm.new_resource_with(&ctx, &realm, "GeneralLedgerPostingRules")?;
```

Creating claims for resources
```rust
let cd_deposit = pm.new_claim_with(&ctx, &realm, &deposit_account, "(CREATE|DELETE)")?;
let ru_deposit = pm.new_claim_with(&ctx, &realm, &deposit_account, "(READ|UPDATE)")?;

let cd_loan = pm.new_claim_with(&ctx, &realm, &loan_account, "(CREATE|DELETE)")?;
let ru_loan = pm.new_claim_with(&ctx, &realm, &loan_account, "(READ|UPDATE)")?;

let rd_ledger = pm.new_claim_with(&ctx, &realm, &general_ledger, "(READ|CREATE)")?;
let r_glpr = pm.new_claim_with(&ctx, &realm, &general_ledger, "(READ)")?;

let cud_glpr = pm.new_claim_with(&ctx, &realm, &posting_rules, "(CREATE|UPDATE|DELETE)")?;
```

Mapping Principals and Claims to Roles
```rust
pm.map_principal_to_role(&ctx, &tom, &teller);
pm.map_principal_to_role(&ctx, &cassy, &csr);
pm.map_principal_to_role(&ctx, &ali, &accountant);
pm.map_principal_to_role(&ctx, &mike, &accountant_manager);
pm.map_principal_to_role(&ctx, &larry, &loan_officer);
```

Map claims to roles as follows:
```rust
pm.map_role_to_claim(&ctx, &teller, &ru_deposit, "U.S.", r#"employeeRegion == "Midwest""#);
pm.map_role_to_claim(&ctx, &csr, &cd_deposit, "U.S.", r#"employeeRegion == "Midwest""#);
pm.map_role_to_claim(&ctx, &accountant, &rd_ledger, "U.S.", r#"employeeRegion == "Midwest" && ledgerYear == current_year()"#);
pm.map_role_to_claim(&ctx, &accountant, &ru_loan, "U.S.", r#"employeeRegion == "Midwest" && accountBlance < 10000"#);
pm.map_role_to_claim(&ctx, &accountant_manager, &cd_loan, "U.S.", r#"employeeRegion == "Midwest" && accountBlance < 10000"#);
pm.map_role_to_claim(&ctx, &accountant_manager, &r_glpr, "U.S.", r#"employeeRegion == "Midwest" && ledgerYear == current_year()"#);
pm.map_role_to_claim(&ctx, &loan_officer, &cud_glpr, "U.S.", r#"employeeRegion == "Midwest" && ledgerYear == current_year()"#);
```

Checking permissions
```rust
let sm = SecurityManager::new(pm);
```

Tom, the teller should be able to READ DepositAccount with scope U.S when employeeRegion == Midwest
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::READ, "DepositAccount", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
assert_eq!(PermissionResult::Allow, sm.check(&req)?);
```

Tom, the teller should not be able to READ DepositAccount with scope U.S when employeeRegion == Northeast
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::READ, "DepositAccount", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Northeast".to_string()));
assert!(sm.check(&req).is_err());
```

Tom, the teller should not be able to DELETE DepositAccount with scope U.S when employeeRegion == Midwest
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::DELETE, "DepositAccount", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
assert!(sm.check(&req).is_err());
```

Cassy, the CSR should be able to DELETE DepositAccount with scope U.S when employeeRegion == Midwest
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), cassy.id.as_str(), ActionType::DELETE, "DepositAccount", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
assert_eq!(PermissionResult::Allow, sm.check(&req)?);
```

Cassy, the CSR should be able to DELETE DepositAccount with scope U.K when employeeRegion == Midwest
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), cassy.id.as_str(), ActionType::DELETE, "DepositAccount", "U.K.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
assert!(sm.check(&req).is_err());
```

Ali, the Accountant should be able to READ GeneralLedger with scope U.S when employeeRegion == Midwest AND ledgerYear == current_year()
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::READ, "GeneralLedger", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
assert_eq!(PermissionResult::Allow, sm.check(&req)?);
```

Ali, the Accountant should not be able to READ GeneralLedger with scope U.S when employeeRegion == Midwest AND ledgerYear is in past
```rust
req.context.add("ledgerYear", ValueWrapper::Int(2000));
assert!(sm.check(&req).is_err());
```

Ali, the Accountant should not be able to DELETE GeneralLedger with scope U.S when employeeRegion == Midwest AND ledgerYear == current_year()
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::DELETE, "GeneralLedger", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
assert!(sm.check(&req).is_err());
```

Mike, the Accountant Manager should be able to DELETE GeneralLedger with scope U.S when employeeRegion == Midwest AND ledgerYear == current_year()
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::CREATE, "GeneralLedger", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
assert_eq!(PermissionResult::Allow, sm.check(&req)?);
```


Mike, the Accountant Manager should not be able to post posting-rules of general-ledger with scope U.S when employeeRegion == Midwest AND ledgerYear == current_year()
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::CREATE, "GeneralLedgerPostingRules", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
req.context.add("accountBlance", ValueWrapper::Int(500));
assert!(sm.check(&req).is_err());
```

Larry, the Loan Officer should be able to post posting-rules of general-ledger with scope U.S when employeeRegion == Midwest AND ledgerYear == current_year()
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), larry.id.as_str(), ActionType::CREATE, "GeneralLedgerPostingRules", "U.S.");
req.context.add("employeeRegion", ValueWrapper::String("Midwest".to_string()));
req.context.add("ledgerYear", ValueWrapper::Int(Utc::now().naive_utc().year() as i64));
req.context.add("accountBlance", ValueWrapper::Int(500));
assert_eq!(PermissionResult::Allow, sm.check(&req)?);
```

### Expense Report

Creating security realm
```rust
let realm = pm.new_realm_with(&ctx, "expense")?;
```

Creating organization
```rust
let org = pm.new_org_with(&ctx, "box-air")?;
```

Creating Groups
```rust
let group_employee = pm.new_group_with(&ctx, &org, "Employee")?;
let group_manager = pm.new_group_with_parent(&ctx, &org, &group_employee, "Manager")?;
```

Creating Users
```rust
let tom = pm.new_principal_with(&ctx, &org, "tom")?;
let mike = pm.new_principal_with(&ctx, &org, "mike")?;
```

Mapping users to groups
```rust
pm.map_principal_to_group(&ctx, &tom, &group_employee);
pm.map_principal_to_group(&ctx, &mike, &group_employee);
pm.map_principal_to_group(&ctx, &mike, &group_manager);
```

Creating Roles
```rust
let employee = pm.new_role_with(&ctx, &realm, &org, "Employee")?;
let manager = pm.new_role_with_parent(&ctx, &realm, &org, &employee, "Manager")?;
```

Creating Resources
```rust
let expense_report = pm.new_resource_with(&ctx, &realm, "ExpenseReport")?;
```

Creating claims for resources
```rust
let submit_report = pm.new_claim_with(&ctx, &realm, &expense_report, "(SUBMIT|VIEW)")?;
let approve_report = pm.new_claim_with(&ctx, &realm, &expense_report, "APPROVE")?;
```

Mapping Principals and Claims to Roles
```rust
pm.map_group_to_role(&ctx, &group_employee, &employee, "");
pm.map_group_to_role(&ctx, &group_manager, &manager, "");
```

Map claims to roles as follows:
```rust
pm.map_role_to_claim(&ctx, &employee, &submit_report, "U.S.", r#"amount < 10000"#);
pm.map_role_to_claim(&ctx, &manager, &approve_report, "U.S.", r#"amount < 10000"#);
```

Checking Permissions
```rust
let sm = SecurityManager::new(pm);
```

Tom should be able to submit report
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::SUBMIT, "ExpenseReport", "U.S.");
req.context.add("amount", ValueWrapper::Int(1000));
assert_eq!(PermissionResult::Allow, sm.check(&req)?);
```

Tom should not be able to approve report
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::APPROVE, "ExpenseReport", "U.S.");
req.context.add("amount", ValueWrapper::Int(1000));
assert!(sm.check(&req).is_err());
```

Mike should be able to approve report
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::APPROVE, "ExpenseReport", "U.S.");
req.context.add("amount", ValueWrapper::Int(1000));
assert_eq!(PermissionResult::Allow, sm.check(&req)?);
```

### Feature flag with Geo-Fencing

Initialize context and repository
```rust
let ctx = SecurityContext::new("0".into(), "0".into());
let cf = DefaultConnectionFactory::new();
let locator = RepositoryFactory::new(&cf);
let pm = locator.new_persistence_manager();
```

Creating security realm
```rust
let realm = pm.new_realm_with(&ctx, "ada")?;
```

Creating organization
```rust
let org = pm.new_org_with(&ctx, "ada")?;
```

Creating Users
```rust
let tom = pm.new_principal_with(&ctx, &org, "tom")?;
let mike = pm.new_principal_with(&ctx, &org, "mike")?;
```

Creating Roles
```rust
let customer = pm.new_role_with(&ctx, &realm, &org, "Customer")?;
let beta_customer = pm.new_role_with_parent(&ctx, &realm, &org, &customer, "BetaCustomer")?;
```

Creating Resources
```rust
let feature = pm.new_resource_with(&ctx, &realm, "Feature")?;
```

Creating claims for resources
```rust
let view = pm.new_claim_with(&ctx, &realm, &feature, "VIEW")?;
```

Mapping Principals and Claims to Roles
```rust
pm.map_principal_to_role(&ctx, &tom, &customer);
pm.map_principal_to_role(&ctx, &mike, &beta_customer);
```

Map claims to roles as follows:
```rust
pm.map_role_to_claim(&ctx, &customer, &view, "UI::Flag::BasicReport", r#"geo_distance_km(customer_lat, customer_lon, 47.620422, -122.349358) < 100"#);
pm.map_role_to_claim(&ctx, &beta_customer, &view, "UI::Flag::AdvancedReport", r#"geo_distance_km(customer_lat, customer_lon, 47.620422, -122.349358) < 200"#);
```

Tom should be able to view basic report if he lives close to Seattle
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::BasicReport");
req.context.add("customer_lat", ValueWrapper::Float(46.879967));
req.context.add("customer_lon", ValueWrapper::Float(-121.726906));
assert_eq!(PermissionResponse::Allow, sm.check(&req)?);
```

Tom should not be able to view basic report if he lives far from Seattle
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::BasicReport");
req.context.add("customer_lat", ValueWrapper::Float(37.3230));
req.context.add("customer_lon", ValueWrapper::Float(-122.0322));
assert!(sm.check(&req).is_err());
```

Tom should not be able to view advanced report
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), tom.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
req.context.add("customer_lat", ValueWrapper::Float(46.879967));
req.context.add("customer_lon", ValueWrapper::Float(-121.726906));
assert!(sm.check(&req).is_err());
```

Mike should be able to view advanced report
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
req.context.add("customer_lat", ValueWrapper::Float(46.879967));
req.context.add("customer_lon", ValueWrapper::Float(-121.726906));
assert_eq!(PermissionResponse::Allow, sm.check(&req)?);
```

Mike should not be able to view advanced report if he lives far from Seattle
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), mike.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
req.context.add("customer_lat", ValueWrapper::Float(37.3230));
req.context.add("customer_lon", ValueWrapper::Float(-122.0322));
assert!(sm.check(&req).is_err());
```

### Using License Policy to restrict access to different level of features

// Creating organization
```rust
let freemium_org = pm.new_org_with(&ctx, "Freeloader")?;
let paid_org = pm.new_org_with(&ctx, "Moneymaker")?;
```

Create license policies
```rust
let freemium_policy = pm.new_license_policy(&ctx, &freemium_org)?;
let paid_policy = pm.new_license_policy(&ctx, &paid_org)?;
```

Creating Users
```rust
let freemium_frank = pm.new_principal_with(&ctx, &freemium_org, "frank")?;
let money_matt = pm.new_principal_with(&ctx, &paid_org, "matt")?;
```

Creating Roles
```rust
let customer = pm.new_role_with(&ctx, &realm, &freemium_org, "Customer")?;
let paid_customer = pm.new_role_with(&ctx, &realm, &paid_org, "PaidCustomer")?;
```

Creating Resources
```rust
let feature = pm.new_resource_with(&ctx, &realm, "Feature")?;
```

Creating claims for resources
```rust
let view = pm.new_claim_with(&ctx, &realm, &feature, "VIEW")?;
```

Mapping Principals and Claims to Roles
```rust
pm.map_principal_to_role(&ctx, &freemium_frank, &customer);
pm.map_principal_to_role(&ctx, &money_matt, &customer);
pm.map_principal_to_role(&ctx, &money_matt, &paid_customer);
```

Map claims to policies as follows:
```rust
pm.map_license_policy_to_claim(&ctx, &freemium_policy, &view, "UI::Flag::BasicReport", "");
pm.map_license_policy_to_claim(&ctx, &paid_policy, &view, "UI::Flag::AdvancedReport", "");
```

Map claims to roles as follows:
```rust
pm.map_role_to_claim(&ctx, &customer, &view, "UI::Flag::BasicReport", "");
pm.map_role_to_claim(&ctx, &paid_customer, &view, "UI::Flag::AdvancedReport", "");
```

Frank should be able to view basic report
```rust
let req = PermissionRequest::new(realm.id.as_str(), freemium_frank.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::BasicReport");
assert_eq!(PermissionResponse::Allow, sm.check(&req)?);
```

Frank should not be able to view advanced report
```rust
let req = PermissionRequest::new(realm.id.as_str(), freemium_frank.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
assert!(sm.check(&req).is_err());
```

Matt should be able to view advanced report
```rust
let req = PermissionRequest::new(realm.id.as_str(), money_matt.id.as_str(), ActionType::VIEW, "Feature", "UI::Flag::AdvancedReport");
assert_eq!(PermissionResponse::Allow, sm.check(&req)?);
```
### Using Multiple teams for roles
Create license policies
```rust
let policy = pm.new_license_policy(&ctx, &org)?;
```

Creating Users
```rust
let dave = pm.new_principal_with(&ctx, &org, "dave")?;
let qari = pm.new_principal_with(&ctx, &org, "qari")?;
let ali = pm.new_principal_with(&ctx, &org, "ali")?;
```

Creating Roles
```rust
let developer = pm.new_role_with(&ctx, &realm, &org, "Developer")?;
let qa = pm.new_role_with(&ctx, &realm, &org, "QA")?;
let admin = pm.new_role_with_parent(&ctx, &realm, &org, &developer, "Admin")?;
```

Creating Resources
```rust
let app = pm.new_resource_with(&ctx, &realm, "App")?;
```

Creating claims for resources
```rust
let submit_view = pm.new_claim_with(&ctx, &realm, &app, "(SUBMIT|VIEW)")?;
let view = pm.new_claim_with(&ctx, &realm, &app, "VIEW")?;
let create_delete = pm.new_claim_with(&ctx, &realm, &app, "(CREATE|DELETE)")?;
```

Mapping Principals and Claims to Roles
```rust
pm.map_principal_to_role(&ctx, &dave, &developer);
pm.map_principal_to_role(&ctx, &qari, &qa);
pm.map_principal_to_role(&ctx, &ali, &admin);
```

Map claims to policies as follows:
```rust
pm.map_license_policy_to_claim(&ctx, &policy, &submit_view, "com.xyz.app", "appSize < 1000");
pm.map_license_policy_to_claim(&ctx, &policy, &view, "com.xyz.app", "appSize < 1000");
pm.map_license_policy_to_claim(&ctx, &policy, &create_delete, "com.xyz.app", "");
```

Map claims to roles as follows:
```rust
pm.map_role_to_claim(&ctx, &developer, &submit_view, "com.xyz.app", "appSize < 1000");
pm.map_role_to_claim(&ctx, &qa, &view, "com.xyz.app", "appSize < 1000");
pm.map_role_to_claim(&ctx, &admin, &create_delete, "com.xyz.app", "");
```

Dave should be able to submit app
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), dave.id.as_str(), ActionType::SUBMIT, "App", "com.xyz.app");
req.context.add("appSize", ValueWrapper::Int(500));
assert_eq!(PermissionResponse::Allow, sm.check(&req)?);
```

Qari should be able to view app
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), qari.id.as_str(), ActionType::VIEW, "App", "com.xyz.app");
req.context.add("appSize", ValueWrapper::Int(500));
assert_eq!(PermissionResponse::Allow, sm.check(&req)?);
```

Qari should not be able to create app
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), qari.id.as_str(), ActionType::CREATE, "App", "com.xyz.app");
req.context.add("appSize", ValueWrapper::Int(500));
assert!(sm.check(&req).is_err());
```

Ali should be able to create app
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::CREATE, "App", "com.xyz.app");
req.context.add("appSize", ValueWrapper::Int(500));
assert_eq!(PermissionResponse::Allow, sm.check(&req)?);
```

Ali should be able to submit app
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::SUBMIT, "App", "com.xyz.app");
req.context.add("appSize", ValueWrapper::Int(500));
assert_eq!(PermissionResponse::Allow, sm.check(&req)?);
```

Ali should not be able to submit app with large app
```rust
let mut req = PermissionRequest::new(realm.id.as_str(), ali.id.as_str(), ActionType::SUBMIT, "App", "com.xyz.app");
req.context.add("appSize", ValueWrapper::Int(5000));
assert!(sm.check(&req).is_err());
```

### Managing Resource Quota

Creating security realm
```rust
let realm = pm.new_realm_with(&ctx, "JobGrid").unwrap();
```

Creating organization
```rust
let abc_corp = pm.new_org_with(&ctx, "ABC").unwrap();
let xyz_corp = pm.new_org_with(&ctx, "XYZ").unwrap();
```

Create license policies
```rust
let abc_policy = pm.new_license_policy(&ctx, &abc_corp).unwrap();
let xyz_policy = pm.new_license_policy(&ctx, &xyz_corp).unwrap();
```

Creating Resources
```rust
let project = pm.new_resource_with(&ctx, &realm, "Project").unwrap();
let job = pm.new_resource_with(&ctx, &realm, "Job").unwrap();
```

Set Resource Quota for abc corp to setup 1 project and 2 jobs

```rust
assert!(pm.new_resource_quota_with(&ctx, &project, &abc_policy, "ABC Project", 1).is_ok());
assert!(pm.new_resource_quota_with(&ctx, &job, &abc_policy, "ABC Jobs", 2).is_ok());
```

Set Resource Quota for xyz corp to setup 2 project and 3 jobs
```rust
assert!(pm.new_resource_quota_with(&ctx, &project, &xyz_policy, "XYZ Project", 2).is_ok());
assert!(pm.new_resource_quota_with(&ctx, &job, &xyz_policy, "XYZ Jobs", 3).is_ok());
```

abc corp can have at most 1 project
```rust
assert!(pm.new_resource_instance_with(&ctx, &project, &abc_policy, "ABC Project", "1", Status::COMPLETED).is_ok());
assert!(pm.new_resource_instance_with(&ctx, &project, &abc_policy, "ABC Project", "2", Status::COMPLETED).is_err());
```

abc corp can have at most 2 jobs
```rust
assert!(pm.new_resource_instance_with(&ctx, &job, &abc_policy, "ABC Jobs", "1", Status::COMPLETED).is_ok());
assert!(pm.new_resource_instance_with(&ctx, &job, &abc_policy, "ABC Jobs", "2", Status::COMPLETED).is_ok());
assert!(pm.new_resource_instance_with(&ctx, &job, &abc_policy, "ABC Jobs", "3", Status::COMPLETED).is_err());
```

xyz corp can have at most 2 project
```rust
assert!(pm.new_resource_instance_with(&ctx, &project, &xyz_policy, "XYZ Project", "1", Status::COMPLETED).is_ok());
assert!(pm.new_resource_instance_with(&ctx, &project, &xyz_policy, "XYZ Project", "2", Status::COMPLETED).is_ok());
assert!(pm.new_resource_instance_with(&ctx, &project, &xyz_policy, "XYZ Project", "3", Status::COMPLETED).is_err());
```

xyz corp can have at most 3 jobs
```rust
assert!(pm.new_resource_instance_with(&ctx, &job, &xyz_policy, "XYZ Jobs", "1", Status::COMPLETED).is_ok());
assert!(pm.new_resource_instance_with(&ctx, &job, &xyz_policy, "XYZ Jobs", "2", Status::COMPLETED).is_ok());
assert!(pm.new_resource_instance_with(&ctx, &job, &xyz_policy, "XYZ Jobs", "3", Status::COMPLETED).is_ok());
assert!(pm.new_resource_instance_with(&ctx, &job, &xyz_policy, "XYZ Jobs", "4", Status::COMPLETED).is_err());
```


## REST APIs
You can start REST API as follows:
```
export RUST_LOG="warn"
cargo run
```

Following are major APIs:

### Realms

   * Query realms: GET /api/realms
   * Create realm: POST /api/realms
   * Update realm: PUT /api/realms/<id>
   * Find realm: GET /api/realms/<id>
   * Delete realm: DELETE /api/realms/<id>

### Resources

   * Query resources: GET /api/realms/<realm_id>/resources
   * Create resource: POST /api/realms/<realm_id>/resources
   * Update resource: PUT /api/realms/<realm_id>/resources/<id>
   * Find resource: GET /api/realms/<realm_id>/resources/<id>
   * Delete resource: DELETE /api/realms/<realm_id>/resources/<id>

### Resource Quota

   * Query quota: GET /api/realms/<realm_id>/resources/<resource_id>/quota
   * Create quota: POST /api/realms/<realm_id>/resources/<resource_id>/quota
   * Update quota: PUT /api/realms/<realm_id>/resources/<resource_id>/quota/<id>
   * Find quota: GET /api/realms/<realm_id>/resources/<resource_id>/quota/<id>
   * Delete quota: DELETE /api/realms/<realm_id>/resources/<resource_id>/quota/<id>

### Resource Instances

  * Query instances: GET /api/realms/<realm_id>/resources/<resource_id>/instances
  * Create resource instance: POST /api/realms/<realm_id>/resources/<resource_id>/instances
  * Update resource instance: PUT /api/realms/<realm_id>/resources/<resource_id>/instances/<id>
  * Find resource instance: GET /api/realms/<realm_id>/resources/<resource_id>/instances/<id>
  * Delete resource instance: DELETE /api/realms/<realm_id>/resources/<resource_id>/instances/<id>

### Claims

  * Query claims within realm: GET /api/realms/<realm_id>/claims
  * Query claims within resource: GET /api/realms/<realm_id>/resources/<resource_id>/claims
  * Create claim: POST /api/realms/<realm_id>/resources/<resource_id>/claims
  * Update claim: PUT /api/realms/<realm_id>/resources/<resource_id>/claims/<id>
  * Find claim: GET /api/realms/<realm_id>/resources/<resource_id>/claims/<id>
  * Delete claim: DELETE /api/realms/<realm_id>/resources/<resource_id>/claims/<id>
  * Add principal to claim: PUT /api/realms/<realm_id>/resources/<resource_id>/claims/<claim_id>/principals/<principal_id>
  * Delete principal from claim: DELETE /api/realms/<realm_id>/resources/<resource_id>/claims/<claim_id>/principals/<principal_id>
  *  Add role to claim: PUT /api/realms/<realm_id>/resources/<resource_id>/claims/<claim_id>/roles/<role_id>
  * Delete role from claim: DELETE /api/realms/<realm_id>/resources/<resource_id>/claims/<claim_id>/roles/<role_id>
  * Add claim to license policy: PUT /api/realms/<realm_id>/resources/<resource_id>/claims/<claim_id>/licenses/<license_policy_id>
  * Remove claim from license policy: DELETE /api/realms/<realm_id>/resources/<resource_id>/claims/<claim_id>/licenses/<license_policy_id>

### Organizations

  * Query all organizations: GET /api/orgs
  * Create organization: POST /api/orgs
  * Update organization: PUT /api/orgs/<id>
  * Find organization: GET /api/orgs/<id>
  * Delete organization: DELETE /api/orgs/<id>

### Groups

  * Query all groups: GET /api/orgs/<org_id>/groups
  * Create group: POST /api/orgs/<org_id>/groups
  * Update group: PUT /api/orgs/<org_id>/groups/<id>
  * Find group: GET /api/orgs/<org_id>/groups/<id>
  * Delete group: DELETE /api/orgs/<org_id>/groups/<id>
  * Add principal to group: PUT /api/orgs/<org_id>/groups/<group_id>/principals/<principal_id>
  * Remove principal from group: DELETE /api/orgs/<org_id>/groups/<group_id>/principals/<principal_id>

### Roles

  * Query all roles: GET /api/orgs/<org_id>/roles
  * Create role: POST /api/orgs/<org_id>/roles
  * Update role: PUT /api/orgs/<org_id>/roles/<id>
  * Find role: GET /api/orgs/<org_id>/roles/<id>
  * Delete role: DELETE /api/orgs/<org_id>/roles/<id>
  * Add principal to role: PUT /api/orgs/<org_id>/roles/<role_id>/principals/<principal_id>
  * Remove principal from role: DELETE /api/orgs/<org_id>/roles/<role_id>/principals/<principal_id>
  * Add group to role: PUT /api/orgs/<org_id>/roles/<role_id>/groups/<group_id>
  * Remove group from role: DELETE /api/orgs/<org_id>/roles/<role_id>/groups/<group_id>

### Principals

  * Query all principals: GET /api/orgs/<org_id>/principals
  * Create principal: POST /api/orgs/<org_id>/principals
  * Update principal: PUT /api/orgs/<org_id>/principals/<id>
  * Find principal: GET /api/orgs/<org_id>/principals/<id>
  * Delete principal: DELETE /api/orgs/<org_id>/principals/<id>

### License Polcies

  * Query license policies: GET /api/orgs/<org_id>/licenses
  * Create license policy: POST /api/orgs/<org_id>/licenses
  * Update license policy: PUT /api/orgs/<org_id>/licenses/<id>
  * Find license policy: GET /api/orgs/<org_id>/licenses/<id>
  * Delete license policy: DELETE /api/orgs/<org_id>/licenses/<id>



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
