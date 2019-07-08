CREATE TABLE IF NOT EXISTS rbac_license_policies (
  id VARCHAR(36) NOT NULL PRIMARY KEY,
  organization_id VARCHAR(36) NOT NULL,
  name VARCHAR(36) NOT NULL,
  description TEXT,
  effective_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expired_at TIMESTAMP NOT NULL,
  created_by VARCHAR(36),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_by VARCHAR(36),
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT rbac_license_policies_org_fk FOREIGN KEY (organization_id)
        REFERENCES rbac_organizations(id)
);

CREATE INDEX IF NOT EXISTS rbac_license_policies_name_ndx ON rbac_license_policies(name);
CREATE INDEX IF NOT EXISTS rbac_license_policies_org_ndx ON rbac_license_policies(organization_id);

