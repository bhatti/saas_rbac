import unittest
import base_test
import json

class PrincipalTest(base_test.BaseTest):
    def setUp(self):
        super(PrincipalTest, self).setUp()
        self._org = self.post('/api/orgs', {"name":"principal_org", "url":"https://myorg.com"})

    def tearDown(self):
        self.delete('/api/orgs/%s' % self._org["id"])
        self.delete('/api/orgs/%s/principals/%s' % (self._org["id"], self._principal["id"]))

    def test_all(self):
        self._principal = self.post('/api/orgs/%s/principals' % self._org["id"], {"username":"my_principal", "organization_id":self._org["id"]})
        self.assertEquals("my_principal", self._principal["username"])
        #
        principals = self.get('/api/orgs/%s/principals' % self._org["id"])
        self.assertEquals(1, len(principals))

    def test_create(self):
        self._principal = self.post('/api/orgs/%s/principals' % self._org["id"], {"username":"my_principal", "organization_id":self._org["id"]})
        self.assertEquals("my_principal", self._principal["username"])
        #
        principal = self.get('/api/orgs/%s/principals/%s' % (self._org["id"], self._principal["id"]))
        self.assertEquals("my_principal", principal["username"])

    def test_update(self):
        self._principal = self.post('/api/orgs/%s/principals' % self._org["id"], {"username":"my_principal", "organization_id":self._org["id"]})
        self.assertEquals("my_principal", self._principal["username"])
        #
        principal = self.put('/api/orgs/%s/principals/%s' % (self._org["id"], self._principal["id"]), {"username":"test_my_principal", "organization_id":self._org["id"], "description": "my desc"})
        self.assertEquals("my_principal", principal["username"])
        self.assertEquals("my desc", principal["description"])

if __name__ == '__main__':
    unittest.main()
