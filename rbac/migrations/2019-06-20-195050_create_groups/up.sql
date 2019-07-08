CREATE TABLE IF NOT EXISTS rbac_groups (
  id VARCHAR(36) NOT NULL PRIMARY KEY,
  parent_id VARCHAR(36),
  organization_id VARCHAR(36) NOT NULL,
  name VARCHAR(150) NOT NULL,
  description TEXT,
  created_by VARCHAR(36),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_by VARCHAR(36),
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT rbac_group_parent_fk FOREIGN KEY (parent_id)
        REFERENCES rbac_groups(id),
  CONSTRAINT rbac_group_org_fk FOREIGN KEY (organization_id)
        REFERENCES rbac_organizations(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS rbac_groups_name_ndx ON rbac_groups(name, organization_id);
CREATE INDEX IF NOT EXISTS rbac_groups_parent_ndx ON rbac_groups(parent_id, organization_id);

INSERT INTO rbac_groups VALUES('00000000-0000-0000-0000-000000000000', NULL, '00000000-0000-0000-0000-000000000000', 'default', 'Default', '00000000-0000-0000-0000-000000000000', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000', CURRENT_TIMESTAMP);
