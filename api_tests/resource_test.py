import unittest
import base_test
import json

class ResourceTest(base_test.BaseTest):
    def setUp(self):
        super(ResourceTest, self).setUp()
        self._realm = self.post('/api/realms', {"id":"resource_realm"})

    def tearDown(self):
        self.delete('/api/realms/%s' % self._realm["id"])
        self.delete('/api/realms/%s/resources/%s' % (self._realm["id"], self._resource["id"]))

    def test_all(self):
        self._resource = self.post('/api/realms/%s/resources' % self._realm["id"], {"resource_name":"my_resource", "realm_id":self._realm["id"]})
        self.assertEquals("my_resource", self._resource["resource_name"])
        #
        resources = self.get('/api/realms/%s/resources' % self._realm["id"])
        self.assertEquals(1, len(resources))

    def test_create(self):
        self._resource = self.post('/api/realms/%s/resources' % self._realm["id"], {"resource_name":"my_resource", "realm_id":self._realm["id"]})
        self.assertEquals("my_resource", self._resource["resource_name"])
        #
        resource = self.get('/api/realms/%s/resources/%s' % (self._realm["id"], self._resource["id"]))
        self.assertEquals("my_resource", resource["resource_name"])

    def test_update(self):
        self._resource = self.post('/api/realms/%s/resources' % self._realm["id"], {"resource_name":"my_resource", "realm_id":self._realm["id"]})
        self.assertEquals("my_resource", self._resource["resource_name"])
        #
        resource = self.put('/api/realms/%s/resources/%s' % (self._realm["id"], self._resource["id"]), {"resource_name":"my_resource", "realm_id":self._realm["id"], "description": "my desc"})
        self.assertEquals("my_resource", resource["resource_name"])
        self.assertEquals("my desc", resource["description"])

if __name__ == '__main__':
    unittest.main()
