CREATE TABLE IF NOT EXISTS rbac_claims (
  id VARCHAR(36) NOT NULL PRIMARY KEY,
  realm_id VARCHAR(100) NOT NULL,
  resource_id VARCHAR(36) NOT NULL,
  action VARCHAR(100) NOT NULL,
  effect VARCHAR(50) NOT NULL DEFAULT 'allow',
  description TEXT,
  created_by VARCHAR(36),
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_by VARCHAR(36),
  updated_at TIMESTAMP  NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT rbac_claims_claim_fk FOREIGN KEY (realm_id)
        REFERENCES rbac_realms(id),
  CONSTRAINT rbac_claims_resource_fk FOREIGN KEY (resource_id)
        REFERENCES rbac_resources(id)
);

CREATE UNIQUE INDEX IF NOT EXISTS rbac_claims_resource_ndx ON rbac_claims(realm_id, resource_id, action);

CREATE TABLE IF NOT EXISTS rbac_claim_claimables(
  claim_id VARCHAR(36) NOT NULL,
  claimable_id VARCHAR(36) NOT NULL,
  claimable_type VARCHAR(100) NOT NULL,
  scope VARCHAR(200) NOT NULL,
  claim_constraints TEXT NOT NULL,
  effective_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expired_at TIMESTAMP NOT NULL,
  created_by VARCHAR(36),
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_by VARCHAR(36),
  updated_at TIMESTAMP  NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CONSTRAINT rbac_claim_claimables_claim_fk FOREIGN KEY (claim_id)
        REFERENCES rbac_claims(id),
  PRIMARY KEY (claim_id, claimable_id, claimable_type)
);

CREATE INDEX IF NOT EXISTS rbac_claim_claimables_ndx ON rbac_claim_claimables(claim_id, claimable_id, claimable_type);
CREATE INDEX IF NOT EXISTS rbac_claim_claimables_date_ndx ON rbac_claim_claimables(claim_id, claimable_id, claimable_type, effective_at, expired_at);
