import unittest
import base_test
import json

class PrincipalGroupTest(base_test.BaseTest):
    def setUp(self):
        super(PrincipalGroupTest, self).setUp()
        self._org = self.post('/api/orgs', {"name":"group_org", "url":"https://myorg.com"})
        self._principal = self.post('/api/orgs/%s/principals' % self._org["id"], {"username":"my_principal", "organization_id":self._org["id"]})
        self._realm = self.post('/api/realms', {"id":"resource_realm"})
        self._group = self.post('/api/orgs/%s/groups' % self._org["id"], {"name":"my_group", "organization_id":self._org["id"]})
        self._license = self.post('/api/orgs/%s/licenses' % self._org["id"], {"name":"my_license", "organization_id":self._org["id"], "effective_at": "2019-01-01T00:00:00", "expired_at": "2030-01-01T00:00:00"})


    def tearDown(self):
        self.delete('/api/orgs/%s/licenses/%s' % (self._org["id"], self._license["id"]))
        self.delete('/api/orgs/%s/groups/%s' % (self._org["id"], self._group["id"]))
        self.delete('/api/realms/%s' % self._realm["id"])
        self.delete('/api/orgs/%s' % self._org["id"])
        self.delete('/api/orgs/%s/principals/%s' % (self._org["id"], self._principal["id"]))

    def test_add_remove_principal_to_group(self):
        resp = self.put('/api/orgs/%s/groups/%s/principals/%s' % (self._org["id"], self._group["id"], self._principal["id"]), {})
        self.assertEquals(1, resp, json.dumps(resp))
        resp = self.delete('/api/orgs/%s/groups/%s/principals/%s' % (self._org["id"], self._group["id"], self._principal["id"]))
        self.assertEquals(1, resp, json.dumps(resp))

if __name__ == '__main__':
    unittest.main()
