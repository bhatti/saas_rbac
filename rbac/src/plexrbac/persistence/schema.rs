table! {
    rbac_audit_records (id) {
        id -> Text,
        message -> Text,
        action -> Nullable<Text>,
        context -> Nullable<Text>,
        created_by -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

table! {
    rbac_claim_claimables (claim_id, claimable_id, claimable_type) {
        claim_id -> Text,
        claimable_id -> Text,
        claimable_type -> Text,
        scope -> Text,
        claim_constraints -> Text,
        effective_at -> Timestamp,
        expired_at -> Timestamp,
        created_by -> Nullable<Text>,
        created_at -> Timestamp,
        updated_by -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

table! {
    rbac_claims (id) {
        id -> Text,
        realm_id -> Text,
        resource_id -> Text,
        action -> Text,
        effect -> Text,
        description -> Nullable<Text>,
        created_by -> Nullable<Text>,
        created_at -> Timestamp,
        updated_by -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

table! {
    rbac_group_principals (group_id, principal_id) {
        group_id -> Text,
        principal_id -> Text,
        created_by -> Nullable<Text>,
        created_at -> Timestamp,
        updated_by -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

table! {
    rbac_groups (id) {
        id -> Text,
        parent_id -> Nullable<Text>,
        organization_id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        created_by -> Nullable<Text>,
        created_at -> Timestamp,
        updated_by -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

table! {
    rbac_license_policies (id) {
        id -> Text,
        organization_id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        effective_at -> Timestamp,
        expired_at -> Timestamp,
        created_by -> Nullable<Text>,
        created_at -> Timestamp,
        updated_by -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

table! {
    rbac_organizations (id) {
        id -> Text,
        parent_id -> Nullable<Text>,
        name -> Text,
        url -> Text,
        description -> Nullable<Text>,
        created_by -> Nullable<Text>,
        created_at -> Timestamp,
        updated_by -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

table! {
    rbac_principals (id) {
        id -> Text,
        organization_id -> Text,
        username -> Text,
        description -> Nullable<Text>,
        created_by -> Nullable<Text>,
        created_at -> Timestamp,
        updated_by -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

table! {
    rbac_realms (id) {
        id -> Text,
        description -> Text,
        created_by -> Nullable<Text>,
        created_at -> Timestamp,
        updated_by -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

table! {
    rbac_resource_instances (id) {
        id -> Text,
        resource_id -> Text,
        license_policy_id -> Text,
        scope -> Text,
        ref_id -> Text,
        status -> Text,
        description -> Nullable<Text>,
        created_by -> Nullable<Text>,
        created_at -> Timestamp,
        updated_by -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

table! {
    rbac_resource_quotas (id) {
        id -> Text,
        resource_id -> Text,
        license_policy_id -> Text,
        scope -> Text,
        max_value -> Integer,
        effective_at -> Timestamp,
        expired_at -> Timestamp,
        created_by -> Nullable<Text>,
        created_at -> Timestamp,
        updated_by -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

table! {
    rbac_resources (id) {
        id -> Text,
        realm_id -> Text,
        resource_name -> Text,
        description -> Nullable<Text>,
        allowable_actions -> Nullable<Text>,
        created_by -> Nullable<Text>,
        created_at -> Timestamp,
        updated_by -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

table! {
    rbac_role_roleables (role_id, roleable_id, roleable_type) {
        role_id -> Text,
        roleable_id -> Text,
        roleable_type -> Text,
        role_constraints -> Text,
        effective_at -> Timestamp,
        expired_at -> Timestamp,
        created_by -> Nullable<Text>,
        created_at -> Timestamp,
        updated_by -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

table! {
    rbac_roles (id) {
        id -> Text,
        parent_id -> Nullable<Text>,
        realm_id -> Text,
        organization_id -> Text,
        name -> Text,
        description -> Nullable<Text>,
        created_by -> Nullable<Text>,
        created_at -> Timestamp,
        updated_by -> Nullable<Text>,
        updated_at -> Timestamp,
    }
}

joinable!(rbac_claim_claimables -> rbac_claims (claim_id));
joinable!(rbac_claims -> rbac_realms (realm_id));
joinable!(rbac_claims -> rbac_resources (resource_id));
joinable!(rbac_group_principals -> rbac_groups (group_id));
joinable!(rbac_group_principals -> rbac_principals (principal_id));
joinable!(rbac_groups -> rbac_organizations (organization_id));
joinable!(rbac_license_policies -> rbac_organizations (organization_id));
joinable!(rbac_principals -> rbac_organizations (organization_id));
joinable!(rbac_resource_instances -> rbac_license_policies (license_policy_id));
joinable!(rbac_resource_instances -> rbac_resources (resource_id));
joinable!(rbac_resource_quotas -> rbac_license_policies (license_policy_id));
joinable!(rbac_resource_quotas -> rbac_resources (resource_id));
joinable!(rbac_resources -> rbac_realms (realm_id));
joinable!(rbac_role_roleables -> rbac_roles (role_id));
joinable!(rbac_roles -> rbac_organizations (organization_id));
joinable!(rbac_roles -> rbac_realms (realm_id));

allow_tables_to_appear_in_same_query!(
    rbac_audit_records,
    rbac_claim_claimables,
    rbac_claims,
    rbac_group_principals,
    rbac_groups,
    rbac_license_policies,
    rbac_organizations,
    rbac_principals,
    rbac_realms,
    rbac_resource_instances,
    rbac_resource_quotas,
    rbac_resources,
    rbac_role_roleables,
    rbac_roles,
);
