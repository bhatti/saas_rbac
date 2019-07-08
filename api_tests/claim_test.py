import unittest
import base_test
import json

class ClaimTest(base_test.BaseTest):
    def setUp(self):
        super(ClaimTest, self).setUp()
        self._org = self.post('/api/orgs', {"name":"claim_org", "url":"https://myorg.com"})
        self._principal = self.post('/api/orgs/%s/principals' % self._org["id"], {"username":"my_principal", "organization_id":self._org["id"]})
        self._realm = self.post('/api/realms', {"id":"resource_realm"})
        self._license = self.post('/api/orgs/%s/licenses' % self._org["id"], {"name":"my_license", "organization_id":self._org["id"], "effective_at": "2019-01-01T00:00:00", "expired_at": "2030-01-01T00:00:00"})
        self._resource = self.post('/api/realms/%s/resources' % self._realm["id"], {"resource_name":"my_resource", "realm_id":self._realm["id"]})


    def tearDown(self):
        self.delete('/api/realms/%s/resources/%s/claims/%s' % (self._realm["id"], self._resource["id"], self._claim["id"]))
        self.delete('/api/orgs/%s/licenses/%s' % (self._org["id"], self._license["id"]))
        self.delete('/api/realms/%s/resources/%s' % (self._realm["id"], self._resource["id"]))
        self.delete('/api/realms/%s' % self._realm["id"])
        self.delete('/api/orgs/%s' % self._org["id"])
        self.delete('/api/orgs/%s/principals/%s' % (self._org["id"], self._principal["id"]))

    def test_get_by_resource(self):
        self._claim = self.post('/api/realms/%s/resources/%s/claims' % (self._realm["id"], self._resource["id"]), {"action":"READ", "realm_id":self._realm["id"]})
        self.assertEquals("READ", self._claim["action"])
        #
        resp = self.get('/api/realms/%s/resources/%s/claims' % (self._realm["id"], self._resource["id"]))
        self.assertEquals(1, len(resp), json.dumps(resp))

    def test_get_by_realm(self):
        self._claim = self.post('/api/realms/%s/resources/%s/claims' % (self._realm["id"], self._resource["id"]), {"action":"READ", "realm_id":self._realm["id"]})
        self.assertEquals("READ", self._claim["action"])
        #
        resp = self.get('/api/realms/%s/claims' % (self._realm["id"]))
        self.assertEquals(1, len(resp), json.dumps(resp))

    def test_create(self):
        self._claim = self.post('/api/realms/%s/resources/%s/claims' % (self._realm["id"], self._resource["id"]), {"action":"READ", "realm_id":self._realm["id"]})
        self.assertEquals("READ", self._claim["action"])
        #
        claim = self.get('/api/realms/%s/resources/%s/claims/%s' % (self._realm["id"], self._resource["id"], self._claim["id"]))
        self.assertEquals("READ", self._claim["action"])

    def test_update(self):
        self._claim = self.post('/api/realms/%s/resources/%s/claims' % (self._realm["id"], self._resource["id"]), {"action":"READ", "realm_id":self._realm["id"]})
        self.assertEquals("READ", self._claim["action"])
        #
        claim = self.put('/api/realms/%s/resources/%s/claims/%s' % (self._realm["id"], self._resource["id"], self._claim["id"]), {"action":"WRITE", "realm_id":self._realm["id"]})
        self.assertEquals("WRITE", claim["action"])
        claim = self.get('/api/realms/%s/resources/%s/claims/%s' % (self._realm["id"], self._resource["id"], self._claim["id"]))
        self.assertEquals("WRITE", claim["action"])

if __name__ == '__main__':
    unittest.main()
