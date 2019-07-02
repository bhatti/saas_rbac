CREATE TABLE IF NOT EXISTS rbac_principals (
  id VARCHAR(36) NOT NULL PRIMARY KEY,
  organization_id VARCHAR(36) NOT NULL,
  username VARCHAR(150) NOT NULL,
  description TEXT,
  created_by VARCHAR(36),
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_by VARCHAR(36),
  updated_at TIMESTAMP  NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT rbac_principal_org_fk FOREIGN KEY (organization_id)
        REFERENCES rbac_organizations(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS rbac_principals_name_ndx ON rbac_principals(username, organization_id);

CREATE TABLE IF NOT EXISTS rbac_group_principals (
  group_id VARCHAR(36) NOT NULL,
  principal_id VARCHAR(36) NOT NULL,
  created_by VARCHAR(36),
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_by VARCHAR(36),
  updated_at TIMESTAMP  NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT rbac_group_principals_group_fk FOREIGN KEY (group_id)
        REFERENCES rbac_groups(id),
  CONSTRAINT rbac_group_principals_principal_fk FOREIGN KEY (principal_id)
        REFERENCES rbac_principals(id),
  PRIMARY KEY (group_id, principal_id)
);

CREATE UNIQUE INDEX IF NOT EXISTS rbac_group_principals_ndx ON rbac_group_principals(group_id, principal_id);

INSERT INTO rbac_principals VALUES('00000000-0000-0000-0000-000000000000', '00000000-0000-0000-0000-000000000000', 'root', 'Admin account', 'SEEDED', CURRENT_TIMESTAMP, 'SEEDED', CURRENT_TIMESTAMP);
INSERT INTO rbac_group_principals VALUES('00000000-0000-0000-0000-000000000000', '00000000-0000-0000-0000-000000000000', 'SEEDED', CURRENT_TIMESTAMP, 'SEEDED', CURRENT_TIMESTAMP);
