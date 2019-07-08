import unittest
import base_test
import json

class RoleTest(base_test.BaseTest):
    def setUp(self):
        super(RoleTest, self).setUp()
        self._org = self.post('/api/orgs', {"name":"role_org", "url":"https://myorg.com"})
        self._realm = self.post('/api/realms', {"id":"role_realm"})

    def tearDown(self):
        self.delete('/api/orgs/%s/roles/%s' % (self._org["id"], self._role["id"]))
        self.delete('/api/orgs/%s/roles/%s' % (self._org["id"], self._rolep["id"]))
        self.delete('/api/orgs/%s' % self._org["id"])
        self.delete('/api/realms/%s' % self._realm["id"])

    def test_all(self):
        self._role = self.post('/api/orgs/%s/roles' % self._org["id"], {"name":"my_role", "organization_id":self._org["id"], "realm_id":self._realm["id"]})
        self.assertEquals("my_role", self._role["name"])
        #
        self._rolep = self.post('/api/orgs/%s/roles' % self._org["id"], {"name":"child_role", "organization_id":self._org["id"], "realm_id":self._realm["id"], "parent_id": self._role["id"]})
        self.assertEquals("child_role", self._rolep["name"])
        #
        roles = self.get('/api/orgs/%s/roles' % self._org["id"])
        self.assertEquals(2, len(roles))

    def test_create(self):
        self._role = self.post('/api/orgs/%s/roles' % self._org["id"], {"name":"my_role", "organization_id":self._org["id"], "realm_id":self._realm["id"]})
        self.assertEquals("my_role", self._role["name"])
        #
        self._rolep = self.post('/api/orgs/%s/roles' % self._org["id"], {"name":"child_role", "organization_id":self._org["id"], "realm_id":self._realm["id"], "parent_id": self._role["id"]})
        self.assertEquals("child_role", self._rolep["name"])
        #
        role = self.get('/api/orgs/%s/roles/%s' % (self._org["id"], self._role["id"]))
        self.assertEquals("my_role", role["name"])

    def test_update(self):
        self._role = self.post('/api/orgs/%s/roles' % self._org["id"], {"name":"my_role", "organization_id":self._org["id"], "realm_id":self._realm["id"]})
        self.assertEquals("my_role", self._role["name"])
        #
        self._rolep = self.post('/api/orgs/%s/roles' % self._org["id"], {"name":"child_role", "organization_id":self._org["id"], "realm_id":self._realm["id"], "parent_id": self._role["id"]})
        self.assertEquals("child_role", self._rolep["name"])
        #
        role = self.put('/api/orgs/%s/roles/%s' % (self._org["id"], self._role["id"]), {"name":"my_role", "organization_id":self._org["id"], "realm_id":self._realm["id"], "description": "my desc"})
        self.assertEquals("my_role", role["name"])
        self.assertEquals("my desc", role["description"])

if __name__ == '__main__':
    unittest.main()
