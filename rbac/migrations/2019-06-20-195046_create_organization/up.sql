CREATE TABLE IF NOT EXISTS rbac_organizations (
  id VARCHAR(36) NOT NULL PRIMARY KEY,
  parent_id VARCHAR(36),
  name VARCHAR(150) NOT NULL,
  url VARCHAR(200) NOT NULL,
  description TEXT,
  created_by VARCHAR(36),
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_by VARCHAR(36),
  updated_at TIMESTAMP  NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT rbac_organizations_parent_fk FOREIGN KEY (parent_id)
        REFERENCES rbac_organizations(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS rbac_organizations_name_ndx ON rbac_organizations(name);
CREATE UNIQUE INDEX IF NOT EXISTS rbac_organizations_parent_ndx ON rbac_organizations(parent_id);
INSERT INTO rbac_organizations VALUES('00000000-0000-0000-0000-000000000000', NULL, 'default', 'http://default', 'Default', 'SEEDED', CURRENT_TIMESTAMP, 'SEEDED', CURRENT_TIMESTAMP);
