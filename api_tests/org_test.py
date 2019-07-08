import unittest
import base_test
import json

class OrgTest(base_test.BaseTest):
    # default setup method
    def setUp(self):
        super(OrgTest, self).setUp()

    def tearDown(self):
        self.delete('/api/orgs/%s' % self._org["id"])


    def test_all(self):
        self._org = self.post('/api/orgs', {"name":"my_org", "url":"https://myorg.com"})
        self.assertEquals("my_org", self._org["name"])
        self.assertEquals("https://myorg.com", self._org["url"])
        resp = json.dumps(self.get('/api/orgs'))
        self.assertTrue('"name": "my_org"' in resp)

    def test_create(self):
        self._org = self.post('/api/orgs', {"name":"my_org", "url":"https://myorg.com"})
        self.assertEquals("my_org", self._org["name"])
        self.assertEquals("https://myorg.com", self._org["url"])
        #
        org = self.get('/api/orgs/%s' % self._org["id"])
        self.assertEquals("my_org", org["name"])
        self.assertEquals("https://myorg.com", org["url"])

    def test_update(self):
        self._org = self.post('/api/orgs', {"name":"my_org", "url":"https://myorg.com"})
        org = self.put('/api/orgs/%s' % self._org["id"], {"name":"my_org", "url":"https://myorg.com", "description": "my desc"})
        #
        org = self.get('/api/orgs/%s' % org["id"])
        self.assertEquals("my_org", org["name"])
        self.assertEquals("https://myorg.com", org["url"])
        self.assertEquals("my desc", org["description"])

if __name__ == '__main__':
    unittest.main()
