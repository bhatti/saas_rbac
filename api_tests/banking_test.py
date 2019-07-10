import unittest
import base_test
import json
import urllib

class BankingTest(base_test.BaseTest):
    def setUp(self):
        super(BankingTest, self).setUp()
        self._realm = self.post('/api/realms', {"id":"banking"})
        self._org = self.post('/api/orgs', {"name":"bank-of-flakes", "url":"https://flakes.banky"})
        #self._license = self.post('/api/orgs/%s/licenses' % self._org["id"], {"name":"default", "organization_id":self._org["id"], "effective_at": "2019-01-01T00:00:00", "expired_at": "2030-01-01T00:00:00"})
        # Creating Users
        self._tom = self.post('/api/orgs/%s/principals' % self._org["id"], {"username":"tom", "organization_id":self._org["id"]})
        self._cassy = self.post('/api/orgs/%s/principals' % self._org["id"], {"username":"cassy", "organization_id":self._org["id"]})
        self._ali = self.post('/api/orgs/%s/principals' % self._org["id"], {"username":"ali", "organization_id":self._org["id"]})
        self._mike = self.post('/api/orgs/%s/principals' % self._org["id"], {"username":"mike", "organization_id":self._org["id"]})
        self._larry= self.post('/api/orgs/%s/principals' % self._org["id"], {"username":"larry", "organization_id":self._org["id"]})
        # Creating Roles
        self._employee = self.post('/api/orgs/%s/roles' % self._org["id"], {"name":"employee", "organization_id":self._org["id"], "realm_id":self._realm["id"]})
        self._teller = self.post('/api/orgs/%s/roles' % self._org["id"], {"name":"teller", "organization_id":self._org["id"], "realm_id":self._realm["id"], "parent_id": self._employee["id"]})
        self._csr = self.post('/api/orgs/%s/roles' % self._org["id"], {"name":"csr", "organization_id":self._org["id"], "realm_id":self._realm["id"], "parent_id": self._teller["id"]})
        self._accountant = self.post('/api/orgs/%s/roles' % self._org["id"], {"name":"accountant", "organization_id":self._org["id"], "realm_id":self._realm["id"], "parent_id": self._employee["id"]})
        self._accountant_manager = self.post('/api/orgs/%s/roles' % self._org["id"], {"name":"accountant_manager", "organization_id":self._org["id"], "realm_id":self._realm["id"], "parent_id": self._accountant["id"]})
        self._loan_officer = self.post('/api/orgs/%s/roles' % self._org["id"], {"name":"loan_officer", "organization_id":self._org["id"], "realm_id":self._realm["id"], "parent_id": self._accountant_manager["id"]})
        # Creating Resources
        self._deposit_account = self.post('/api/realms/%s/resources' % self._realm["id"], {"resource_name":"DepositAccount", "realm_id":self._realm["id"]})
        self._loan_account = self.post('/api/realms/%s/resources' % self._realm["id"], {"resource_name":"LoanAccount", "realm_id":self._realm["id"]})
        self._general_ledger = self.post('/api/realms/%s/resources' % self._realm["id"], {"resource_name":"GeneralLedger", "realm_id":self._realm["id"]})
        self._posting_rules = self.post('/api/realms/%s/resources' % self._realm["id"], {"resource_name":"GeneralLedgerPostingRules", "realm_id":self._realm["id"]})

        # Creating claims for resources
        self._cd_deposit = self.post('/api/realms/%s/resources/%s/claims' % (self._realm["id"], self._deposit_account["id"]), {"action":"(CREATE|DELETE)", "realm_id":self._realm["id"]})
        self._ru_deposit = self.post('/api/realms/%s/resources/%s/claims' % (self._realm["id"], self._deposit_account["id"]), {"action":"(READ|UPDATE)", "realm_id":self._realm["id"]})
        self._cd_loan = self.post('/api/realms/%s/resources/%s/claims' % (self._realm["id"], self._loan_account["id"]), {"action":"(CREATE|DELETE)", "realm_id":self._realm["id"]})
        self._ru_loan = self.post('/api/realms/%s/resources/%s/claims' % (self._realm["id"], self._loan_account["id"]), {"action":"(READ|UPDATE)", "realm_id":self._realm["id"]})
        self._rd_ledger = self.post('/api/realms/%s/resources/%s/claims' % (self._realm["id"], self._general_ledger["id"]), {"action":"(READ|DELETE|CREATE)", "realm_id":self._realm["id"]})
        self._r_glpr = self.post('/api/realms/%s/resources/%s/claims' % (self._realm["id"], self._general_ledger["id"]), {"action":"(READ)", "realm_id":self._realm["id"]})
        self._cud_glpr = self.post('/api/realms/%s/resources/%s/claims' % (self._realm["id"], self._posting_rules["id"]), {"action":"(CREATE|UPDATE|DELETE)", "realm_id":self._realm["id"]})

        # Mapping Principals to Roles
        self.put('/api/orgs/%s/roles/%s/principals/%s?max=10&constraints=&expired_at=%s' % (self._org["id"], self._teller["id"], self._tom["id"], urllib.quote('2033-6-17T00:00:00+05:30', safe='')), {})
        self.put('/api/orgs/%s/roles/%s/principals/%s?max=10&constraints=&expired_at=%s' % (self._org["id"], self._csr["id"], self._cassy["id"], urllib.quote('2033-6-17T00:00:00+05:30', safe='')), {})
        self.put('/api/orgs/%s/roles/%s/principals/%s?max=10&constraints=&expired_at=%s' % (self._org["id"], self._accountant["id"], self._ali["id"], urllib.quote('2033-6-17T00:00:00+05:30', safe='')), {})
        self.put('/api/orgs/%s/roles/%s/principals/%s?max=10&constraints=&expired_at=%s' % (self._org["id"], self._accountant_manager["id"], self._mike["id"], urllib.quote('2033-6-17T00:00:00+05:30', safe='')), {})
        self.put('/api/orgs/%s/roles/%s/principals/%s?max=10&constraints=&expired_at=%s' % (self._org["id"], self._loan_officer["id"], self._larry["id"], urllib.quote('2033-6-17T00:00:00+05:30', safe='')), {})

        # Map claims to roles as follows:
        self.put('/api/realms/%s/resources/%s/claims/%s/roles/%s' % (self._realm["id"], self._ru_deposit["resource_id"], self._ru_deposit["id"], self._teller["id"]), {})
        self.put('/api/realms/%s/resources/%s/claims/%s/roles/%s' % (self._realm["id"], self._cd_deposit["resource_id"], self._cd_deposit["id"], self._csr["id"]), {})
        self.put('/api/realms/%s/resources/%s/claims/%s/roles/%s' % (self._realm["id"], self._rd_ledger["resource_id"], self._rd_ledger["id"], self._accountant["id"]), {})
        self.put('/api/realms/%s/resources/%s/claims/%s/roles/%s' % (self._realm["id"], self._ru_loan["resource_id"], self._ru_loan["id"], self._accountant["id"]), {})
        self.put('/api/realms/%s/resources/%s/claims/%s/roles/%s' % (self._realm["id"], self._cd_loan["resource_id"], self._cd_loan["id"], self._accountant_manager["id"]), {})
        self.put('/api/realms/%s/resources/%s/claims/%s/roles/%s' % (self._realm["id"], self._r_glpr["resource_id"], self._r_glpr["id"], self._accountant_manager["id"]), {})
        self.put('/api/realms/%s/resources/%s/claims/%s/roles/%s' % (self._realm["id"], self._cud_glpr["resource_id"], self._cud_glpr["id"], self._loan_officer["id"]), {})

    def tearDown(self):
        # UnMap claims to roles as follows:
        self.delete('/api/realms/%s/resources/%s/claims/%s/roles/%s' % (self._realm["id"], self._ru_deposit["resource_id"], self._ru_deposit["id"], self._teller["id"]))
        self.delete('/api/realms/%s/resources/%s/claims/%s/roles/%s' % (self._realm["id"], self._cd_deposit["resource_id"], self._cd_deposit["id"], self._csr["id"]))
        self.delete('/api/realms/%s/resources/%s/claims/%s/roles/%s' % (self._realm["id"], self._rd_ledger["resource_id"], self._rd_ledger["id"], self._accountant["id"]))
        self.delete('/api/realms/%s/resources/%s/claims/%s/roles/%s' % (self._realm["id"], self._ru_loan["resource_id"], self._ru_loan["id"], self._accountant["id"]))
        self.delete('/api/realms/%s/resources/%s/claims/%s/roles/%s' % (self._realm["id"], self._cd_loan["resource_id"], self._cd_loan["id"], self._accountant_manager["id"]))
        self.delete('/api/realms/%s/resources/%s/claims/%s/roles/%s' % (self._realm["id"], self._r_glpr["resource_id"], self._r_glpr["id"], self._accountant_manager["id"]))
        self.delete('/api/realms/%s/resources/%s/claims/%s/roles/%s' % (self._realm["id"], self._cud_glpr["resource_id"], self._cud_glpr["id"], self._loan_officer["id"]))

        # UnMapping Principals and Claims to Roles
        self.delete('/api/orgs/%s/roles/%s/principals/%s' % (self._org["id"], self._teller["id"], self._tom["id"]))
        self.delete('/api/orgs/%s/roles/%s/principals/%s' % (self._org["id"], self._csr["id"], self._cassy["id"]))
        self.delete('/api/orgs/%s/roles/%s/principals/%s' % (self._org["id"], self._accountant["id"], self._ali["id"]))
        self.delete('/api/orgs/%s/roles/%s/principals/%s' % (self._org["id"], self._accountant_manager["id"], self._mike["id"]))
        self.delete('/api/orgs/%s/roles/%s/principals/%s' % (self._org["id"], self._loan_officer["id"], self._larry["id"]))
        # Deleting claims for resources
        self.delete('/api/realms/%s/resources/%s/claims/%s' % (self._realm["id"], self._cd_deposit["resource_id"], self._cd_deposit["id"]))
        self.delete('/api/realms/%s/resources/%s/claims/%s' % (self._realm["id"], self._ru_deposit["resource_id"], self._ru_deposit["id"]))
        self.delete('/api/realms/%s/resources/%s/claims/%s' % (self._realm["id"], self._cd_loan["resource_id"], self._cd_loan["id"]))
        self.delete('/api/realms/%s/resources/%s/claims/%s' % (self._realm["id"], self._ru_loan["resource_id"], self._ru_loan["id"]))
        self.delete('/api/realms/%s/resources/%s/claims/%s' % (self._realm["id"], self._rd_ledger["resource_id"], self._rd_ledger["id"]))
        self.delete('/api/realms/%s/resources/%s/claims/%s' % (self._realm["id"], self._r_glpr["resource_id"], self._r_glpr["id"]))
        self.delete('/api/realms/%s/resources/%s/claims/%s' % (self._realm["id"], self._cud_glpr["resource_id"], self._cud_glpr["id"]))
        # Deleting Users
        self.delete('/api/orgs/%s/principals/%s' % (self._org["id"], self._tom["id"]))
        self.delete('/api/orgs/%s/principals/%s' % (self._org["id"], self._cassy["id"]))
        self.delete('/api/orgs/%s/principals/%s' % (self._org["id"], self._ali["id"]))
        self.delete('/api/orgs/%s/principals/%s' % (self._org["id"], self._mike["id"]))
        self.delete('/api/orgs/%s/principals/%s' % (self._org["id"], self._larry["id"]))
        # Deleting Roles
        self.delete('/api/orgs/%s/roles/%s' % (self._org["id"], self._employee["id"]))
        self.delete('/api/orgs/%s/roles/%s' % (self._org["id"], self._teller["id"]))
        self.delete('/api/orgs/%s/roles/%s' % (self._org["id"], self._csr["id"]))
        self.delete('/api/orgs/%s/roles/%s' % (self._org["id"], self._accountant ["id"]))
        self.delete('/api/orgs/%s/roles/%s' % (self._org["id"], self._accountant_manager ["id"]))
        self.delete('/api/orgs/%s/roles/%s' % (self._org["id"], self._loan_officer ["id"]))
        # Deleting Resources
        self.delete('/api/realms/%s/resources/%s' % (self._realm["id"], self._deposit_account["id"]))
        self.delete('/api/realms/%s/resources/%s' % (self._realm["id"], self._loan_account["id"]))
        self.delete('/api/realms/%s/resources/%s' % (self._realm["id"], self._general_ledger["id"]))
        self.delete('/api/realms/%s/resources/%s' % (self._realm["id"], self._posting_rules["id"]))
        # Delete realm and org
        self.delete('/api/realms/banking')
        self.delete('/api/orgs/%s' % self._org["id"])
        #self.delete('/api/orgs/%s/licenses/%s' % (self._org["id"], self._license["id"]))

    def test_tom_teller_may_read_deposit_account(self):
        self._principal = self._tom
        resp = self.get('/api/security?resource=DepositAccount&action=READ')
        self.assertEquals("Allow", resp)

    def test_ali_accountant_may_not_read_deposit_account(self):
        self._principal = self._ali
        try:
            resp = self.get('/api/security?resource=DepositAccount&action=READ')
            self.assertTrue(False)
        except Exception as e:
            None

    def test_tom_teller_may_not_delete_deposit_account(self):
        self._principal = self._tom
        try:
            resp = self.get('/api/security?resource=DepositAccount&action=DELETE')
            self.assertTrue(False)
        except Exception as e:
            None

    def test_cassy_csr_may_delete_deposit_account(self):
        self._principal = self._cassy
        resp = self.get('/api/security?resource=DepositAccount&action=DELETE')
        self.assertEquals("Allow", resp)

    def test_ali_accountant_may_read_general_ledger(self):
        self._principal = self._ali
        resp = self.get('/api/security?resource=GeneralLedger&action=READ')
        self.assertEquals("Allow", resp)

    def test_ali_accountant_may_not_delete_general_ledger(self):
        self._principal = self._ali
        try:
            resp = self.get('/api/security?resource=GeneralLedger&action=DELETE')
            self.assertTrue(False)
        except Exception as e:
            None

    def test_mike_accountant_manager_may_delete_general_ledger(self):
        self._principal = self._mike
        resp = self.get('/api/security?resource=GeneralLedger&action=DELETE')
        self.assertEquals("Allow", resp)

    def test_mike_accountant_manager_may_not_post_rules(self):
        self._principal = self._mike
        try:
            resp = self.get('/api/security?resource=GeneralLedgerPostingRules&action=CREATE')
            self.assertTrue(False)
        except Exception as e:
            None

    def test_larry_loan_officerr_may_post_rules(self):
        self._principal = self._larry
        resp = self.get('/api/security?resource=GeneralLedger&action=DELETE')
        self.assertEquals("Allow", resp)

if __name__ == '__main__':
    unittest.main()

