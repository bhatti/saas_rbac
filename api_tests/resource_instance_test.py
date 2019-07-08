import unittest
import base_test
import json

class ResourceInstanceTest(base_test.BaseTest):
    def setUp(self):
        super(ResourceInstanceTest, self).setUp()
        self._org = self.post('/api/orgs', {"name":"resource_instance_org", "url":"https://myorg.com"})
        self._principal = self.post('/api/orgs/%s/principals' % self._org["id"], {"username":"my_principal", "organization_id":self._org["id"]})
        self._realm = self.post('/api/realms', {"id":"resource_realm"})
        self._license = self.post('/api/orgs/%s/licenses' % self._org["id"], {"name":"my_license", "organization_id":self._org["id"], "effective_at": "2019-01-01T00:00:00", "expired_at": "2030-01-01T00:00:00"})
        self._resource = self.post('/api/realms/%s/resources' % self._realm["id"], {"resource_name":"my_resource", "realm_id":self._realm["id"]})
        self._quota = self.post('/api/realms/%s/resources/%s/quota' % (self._realm["id"], self._resource["id"]), {"scope":"scope", "license_policy_id":self._license["id"], "max_value": 3, "effective_at": "2019-01-01T00:00:00", "expired_at": "2030-01-01T00:00:00"})


    def tearDown(self):
        self.delete('/api/realms/%s/resources/%s/instances/%s' % (self._realm["id"], self._resource["id"], self._instance["id"]))
        self.delete('/api/realms/%s/resources/%s/quota/%s' % (self._realm["id"], self._resource["id"], self._quota["id"]))
        self.delete('/api/orgs/%s/licenses/%s' % (self._org["id"], self._license["id"]))
        self.delete('/api/realms/%s/resources/%s' % (self._realm["id"], self._resource["id"]))
        self.delete('/api/realms/%s' % self._realm["id"])
        self.delete('/api/orgs/%s' % self._org["id"])
        self.delete('/api/orgs/%s/principals/%s' % (self._org["id"], self._principal["id"]))

    def test_all(self):
        self._instance = self.post('/api/realms/%s/resources/%s/instances' % (self._realm["id"], self._resource["id"]), {"scope":"scope", "license_policy_id":self._license["id"], "ref_id": "my-ref", "status": "COMPLETED"})
        self.assertEquals("scope", self._instance["scope"])
        #
        instances = self.get('/api/realms/%s/resources/%s/instances' % (self._realm["id"], self._resource["id"]))
        self.assertEquals(1, len(instances), json.dumps(instances))

    def test_create(self):
        self._instance = self.post('/api/realms/%s/resources/%s/instances' % (self._realm["id"], self._resource["id"]), {"scope":"scope", "license_policy_id":self._license["id"], "ref_id": "my-ref", "status": "COMPLETED"})
        self.assertEquals("scope", self._instance["scope"])
        #
        instance = self.get('/api/realms/%s/resources/%s/instances/%s' % (self._realm["id"], self._resource["id"], self._instance["id"]))
        self.assertEquals("scope", instance["scope"])

    def test_update(self):
        self._instance = self.post('/api/realms/%s/resources/%s/instances' % (self._realm["id"], self._resource["id"]), {"scope":"scope", "license_policy_id":self._license["id"], "ref_id": "my-ref", "status": "PENDING"})
        self.assertEquals("scope", self._instance["scope"])
        self.assertEquals("PENDING", self._instance["status"])
        #
        instance = self.put('/api/realms/%s/resources/%s/instances/%s' % (self._realm["id"], self._resource["id"], self._instance["id"]), {"scope":"myscope", "license_policy_id":self._license["id"], "ref_id": "my-ref", "status": "COMPLETED"})
        self.assertEquals("scope", instance["scope"])
        self.assertEquals("COMPLETED", instance["status"])

        instance = self.get('/api/realms/%s/resources/%s/instances/%s' % (self._realm["id"], self._resource["id"], self._instance["id"]))
        self.assertEquals("scope", instance["scope"])
        self.assertEquals("COMPLETED", instance["status"])

if __name__ == '__main__':
    unittest.main()
