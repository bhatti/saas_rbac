import unittest
import base_test
import json

class GroupTest(base_test.BaseTest):
    def setUp(self):
        super(GroupTest, self).setUp()
        self._org = self.post('/api/orgs', {"name":"group_org", "url":"https://myorg.com"})

    def tearDown(self):
        self.delete('/api/orgs/%s' % self._org["id"])
        self.delete('/api/orgs/%s/groups/%s' % (self._org["id"], self._group["id"]))
        self.delete('/api/orgs/%s/groups/%s' % (self._org["id"], self._groupp["id"]))

    def test_all(self):
        self._group = self.post('/api/orgs/%s/groups' % self._org["id"], {"name":"my_group", "organization_id":self._org["id"]})
        self.assertEquals("my_group", self._group["name"])
        #
        self._groupp = self.post('/api/orgs/%s/groups' % self._org["id"], {"name":"child_group", "organization_id":self._org["id"], "parent_id": self._group["id"]})
        self.assertEquals("child_group", self._groupp["name"])
        #
        groups = self.get('/api/orgs/%s/groups' % self._org["id"])
        self.assertEquals(2, len(groups))

    def test_create(self):
        self._group = self.post('/api/orgs/%s/groups' % self._org["id"], {"name":"my_group", "organization_id":self._org["id"]})
        self.assertEquals("my_group", self._group["name"])
        #
        self._groupp = self.post('/api/orgs/%s/groups' % self._org["id"], {"name":"child_group", "organization_id":self._org["id"], "parent_id": self._group["id"]})
        self.assertEquals("child_group", self._groupp["name"])
        #
        group = self.get('/api/orgs/%s/groups/%s' % (self._org["id"], self._group["id"]))
        self.assertEquals("my_group", group["name"])

    def test_update(self):
        self._group = self.post('/api/orgs/%s/groups' % self._org["id"], {"name":"my_group", "organization_id":self._org["id"]})
        self.assertEquals("my_group", self._group["name"])
        #
        self._groupp = self.post('/api/orgs/%s/groups' % self._org["id"], {"name":"child_group", "organization_id":self._org["id"], "parent_id": self._group["id"]})
        self.assertEquals("child_group", self._groupp["name"])
        #
        group = self.put('/api/orgs/%s/groups/%s' % (self._org["id"], self._group["id"]), {"name":"my_group", "organization_id":self._org["id"], "description": "my desc"})
        self.assertEquals("my_group", group["name"])
        self.assertEquals("my desc", group["description"])

if __name__ == '__main__':
    unittest.main()
