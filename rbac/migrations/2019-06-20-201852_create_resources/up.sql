CREATE TABLE IF NOT EXISTS rbac_resources (
  id VARCHAR(36) NOT NULL PRIMARY KEY,
  realm_id VARCHAR(100) NOT NULL,
  resource_name VARCHAR(50) NOT NULL,
  description TEXT,
  allowable_actions TEXT,
  created_by VARCHAR(36),
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_by VARCHAR(36),
  updated_at TIMESTAMP  NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT rbac_resources_realm_fk FOREIGN KEY (realm_id)
        REFERENCES rbac_realms(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS rbac_resources_type_ndx ON rbac_resources(realm_id, resource_name);

CREATE TABLE IF NOT EXISTS rbac_resource_instances (
  id VARCHAR(36) NOT NULL PRIMARY KEY,
  resource_id VARCHAR(36) NOT NULL,
  resourceable_id VARCHAR(36) NOT NULL,
  resourceable_type VARCHAR(50) NOT NULL,
  ref_id VARCHAR(100) NOT NULL,
  status VARCHAR(50) NOT NULL DEFAULT "INFLIGHT",
  description TEXT,
  created_by VARCHAR(36),
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_by VARCHAR(36),
  updated_at TIMESTAMP  NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT rbac_resource_instances_resource_fk FOREIGN KEY (resource_id)
        REFERENCES rbac_resources(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS rbac_resource_insts_ref_ndx ON rbac_resource_instances(resource_id, resourceable_id, resourceable_type, ref_id);

CREATE TABLE IF NOT EXISTS rbac_resource_limits (
  id VARCHAR(36) NOT NULL PRIMARY KEY,
  resource_id VARCHAR(36) NOT NULL,
  limitable_id VARCHAR(36) NOT NULL,
  limitable_type VARCHAR(50) NOT NULL,
  max_value INTEGER NOT NULL DEFAULT 0,
  effective_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expired_at TIMESTAMP NOT NULL,
  created_by VARCHAR(36),
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_by VARCHAR(36),
  updated_at TIMESTAMP  NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT rbac_resource_limits_resources_fk FOREIGN KEY (resource_id)
        REFERENCES rbac_resources(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS rbac_resources_limits_ref_ndx ON rbac_resource_limits(resource_id, limitable_id, limitable_type);
CREATE INDEX IF NOT EXISTS rbac_resources_limits_date_ndx ON rbac_resource_limits(resource_id, limitable_id, limitable_type, effective_at, expired_at);

