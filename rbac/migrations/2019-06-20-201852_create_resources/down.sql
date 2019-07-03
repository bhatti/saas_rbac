DROP INDEX IF EXISTS rbac_resources_type_ndx;
DROP TABLE IF EXISTS rbac_resources;

DROP INDEX IF EXISTS rbac_resource_insts_ref_ndx;
DROP INDEX IF EXISTS rbac_resource_insts_policy_ndx;
DROP TABLE IF EXISTS rbac_resource_instances;

DROP INDEX IF EXISTS rbac_resources_quotas_date_ndx;
DROP INDEX IF EXISTS rbac_resources_quotas_ref_ndx;
DROP TABLE IF EXISTS rbac_resource_quotas;
