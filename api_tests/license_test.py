import unittest
import base_test
import json

class LicenseTest(base_test.BaseTest):
    def setUp(self):
        super(LicenseTest, self).setUp()
        self._org = self.post('/api/orgs', {"name":"license_org", "url":"https://myorg.com"})

    def tearDown(self):
        self.delete('/api/orgs/%s' % self._org["id"])
        self.delete('/api/orgs/%s/licenses/%s' % (self._org["id"], self._license["id"]))

    def test_all(self):
        self._license = self.post('/api/orgs/%s/licenses' % self._org["id"], {"name":"my_license", "organization_id":self._org["id"], "effective_at": "2019-01-01T00:00:00", "expired_at": "2030-01-01T00:00:00"})
        self.assertEquals("my_license", self._license["name"])
        #
        licenses = self.get('/api/orgs/%s/licenses' % self._org["id"])
        self.assertEquals(1, len(licenses))

    def test_create(self):
        self._license = self.post('/api/orgs/%s/licenses' % self._org["id"], {"name":"my_license", "organization_id":self._org["id"], "effective_at": "2019-01-01T00:00:00", "expired_at": "2030-01-01T00:00:00"})
        self.assertEquals("my_license", self._license["name"])
        #
        license = self.get('/api/orgs/%s/licenses/%s' % (self._org["id"], self._license["id"]))
        self.assertEquals("my_license", license["name"])

    def test_update(self):
        self._license = self.post('/api/orgs/%s/licenses' % self._org["id"], {"name":"my_license", "organization_id":self._org["id"], "effective_at": "2019-01-01T00:00:00", "expired_at": "2030-01-01T00:00:00"})
        self.assertEquals("my_license", self._license["name"])
        #
        license = self.put('/api/orgs/%s/licenses/%s' % (self._org["id"], self._license["id"]), {"name":"test_my_license", "organization_id":self._org["id"], "description": "my desc", "effective_at": "2019-01-01T00:00:00", "expired_at": "2030-01-01T00:00:00"})
        self.assertEquals("my_license", license["name"])
        self.assertEquals("my desc", license["description"])

if __name__ == '__main__':
    unittest.main()
