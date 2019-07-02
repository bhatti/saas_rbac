CREATE TABLE IF NOT EXISTS rbac_roles (
  id VARCHAR(36) NOT NULL PRIMARY KEY,
  parent_id VARCHAR(36),
  realm_id VARCHAR(100) NOT NULL,
  organization_id VARCHAR(36) NOT NULL,
  name VARCHAR(150) NOT NULL,
  description TEXT,
  created_by VARCHAR(36),
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_by VARCHAR(36),
  updated_at TIMESTAMP  NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT rbac_role_parent_fk FOREIGN KEY (parent_id)
        REFERENCES rbac_roles(id),
  CONSTRAINT rbac_role_realm_fk FOREIGN KEY (realm_id)
        REFERENCES rbac_realms(id),
  CONSTRAINT rbac_role_org_fk FOREIGN KEY (organization_id)
        REFERENCES rbac_organizations(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS rbac_roles_name_ndx ON rbac_roles(name, realm_id, organization_id);
CREATE INDEX IF NOT EXISTS rbac_roles_parent_ndx ON rbac_roles(parent_id, realm_id, organization_id);

CREATE TABLE IF NOT EXISTS rbac_role_roleables (
  role_id VARCHAR(36) NOT NULL,
  roleable_id VARCHAR(36) NOT NULL,
  roleable_type VARCHAR(50) NOT NULL,
  role_constraints TEXT NOT NULL,
  effective_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expired_at TIMESTAMP NOT NULL,
  created_by VARCHAR(36),
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_by VARCHAR(36),
  updated_at TIMESTAMP  NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT rbac_role_roleables_role_fk FOREIGN KEY (role_id)
        REFERENCES rbac_roles(id),
  PRIMARY KEY (role_id, roleable_id, roleable_type)
);

CREATE UNIQUE INDEX IF NOT EXISTS rbac_role_roleables_ndx ON rbac_role_roleables(role_id, roleable_id, roleable_type);

INSERT INTO rbac_roles VALUES('00000000-0000-0000-0000-000000000000', NULL, '00000000-0000-0000-0000-000000000000', '00000000-0000-0000-0000-000000000000', 'Admin', 'Admin Role', 'SEEDED', CURRENT_TIMESTAMP, 'SEEDED', CURRENT_TIMESTAMP);
INSERT INTO rbac_role_roleables VALUES('00000000-0000-0000-0000-000000000000', '00000000-0000-0000-0000-000000000000', 'Principal', '', CURRENT_TIMESTAMP, DATE('2100-01-01'), 'SEEDED', CURRENT_TIMESTAMP, 'SEEDED', CURRENT_TIMESTAMP);
