import unittest
import base_test
import json

class ResourceQuotaTest(base_test.BaseTest):
    def setUp(self):
        super(ResourceQuotaTest, self).setUp()
        self._org = self.post('/api/orgs', {"name":"resource_quota_org", "url":"https://myorg.com"})
        self._principal = self.post('/api/orgs/%s/principals' % self._org["id"], {"username":"my_principal", "organization_id":self._org["id"]})
        self._realm = self.post('/api/realms', {"id":"resource_realm"})
        self._license = self.post('/api/orgs/%s/licenses' % self._org["id"], {"name":"my_license", "organization_id":self._org["id"], "effective_at": "2019-01-01T00:00:00", "expired_at": "2030-01-01T00:00:00"})
        self._resource = self.post('/api/realms/%s/resources' % self._realm["id"], {"resource_name":"my_resource", "realm_id":self._realm["id"]})


    def tearDown(self):
        self.delete('/api/realms/%s/resources/%s/quota/%s' % (self._realm["id"], self._resource["id"], self._quota["id"]))
        self.delete('/api/orgs/%s/licenses/%s' % (self._org["id"], self._license["id"]))
        self.delete('/api/realms/%s/resources/%s' % (self._realm["id"], self._resource["id"]))
        self.delete('/api/realms/%s' % self._realm["id"])
        self.delete('/api/orgs/%s' % self._org["id"])
        self.delete('/api/orgs/%s/principals/%s' % (self._org["id"], self._principal["id"]))

    def test_all(self):
        self._quota = self.post('/api/realms/%s/resources/%s/quota' % (self._realm["id"], self._resource["id"]), {"scope":"scope", "license_policy_id":self._license["id"], "max_value": 3, "effective_at": "2019-01-01T00:00:00", "expired_at": "2030-01-01T00:00:00"})
        self.assertEquals("scope", self._quota["scope"])
        #
        resp = self.get('/api/realms/%s/resources/%s/quota' % (self._realm["id"], self._resource["id"]))
        self.assertEquals(1, len(resp), json.dumps(resp))

    def test_create(self):
        self._quota = self.post('/api/realms/%s/resources/%s/quota' % (self._realm["id"], self._resource["id"]), {"scope":"scope", "license_policy_id":self._license["id"], "max_value": 3, "effective_at": "2019-01-01T00:00:00", "expired_at": "2030-01-01T00:00:00"})
        self.assertEquals("scope", self._quota["scope"])
        #
        quota = self.get('/api/realms/%s/resources/%s/quota/%s' % (self._realm["id"], self._resource["id"], self._quota["id"]))
        self.assertEquals("scope", quota["scope"])

    def test_update(self):
        self._quota = self.post('/api/realms/%s/resources/%s/quota' % (self._realm["id"], self._resource["id"]), {"scope":"scope", "license_policy_id":self._license["id"], "max_value": 3, "effective_at": "2019-01-01T00:00:00", "expired_at": "2030-01-01T00:00:00"})
        self.assertEquals("scope", self._quota["scope"])
        #
        quota = self.put('/api/realms/%s/resources/%s/quota/%s' % (self._realm["id"], self._resource["id"], self._quota["id"]), {"scope":"scope", "license_policy_id":self._license["id"], "max_value": 8, "effective_at": "2019-01-01T00:00:00", "expired_at": "2030-01-01T00:00:00"})
        self.assertEquals(8, quota["max_value"])

        quota = self.get('/api/realms/%s/resources/%s/quota/%s' % (self._realm["id"], self._resource["id"], self._quota["id"]))
        self.assertEquals(8, quota["max_value"])

if __name__ == '__main__':
    unittest.main()
