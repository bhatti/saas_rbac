import unittest
import base_test
import json

class RealmTest(base_test.BaseTest):
    # default setup method
    def setUp(self):
        super(RealmTest, self).setUp()
        try:
            self.delete('/api/realms/my_realm')
        except Exception as e:
            print "Failed to delete %s\n" % str(e)
            None

    def test_all(self):
        self.post('/api/realms', {"id":"my_realm"})
        json_resp = self.get('/api/realms')
        self.assertTrue('{"id": "my_realm"}' in json.dumps(json_resp))

    def test_create(self):
        json_resp = self.post('/api/realms', {"id":"my_realm"})
        self.assertEquals('{"id": "my_realm"}', json.dumps(json_resp))
        #
        json_resp = self.get('/api/realms/my_realm')
        self.assertEquals('{"id": "my_realm"}', json.dumps(json_resp))

    def test_update(self):
        json_resp = self.post('/api/realms', {"id":"my_realm"})
        json_resp = self.put('/api/realms/my_realm', {"id":"my_realm", "description": "my desc"})
        self.assertEquals('{"id": "my_realm", "description": "my desc"}', json.dumps(json_resp))
        #
        json_resp = self.get('/api/realms/my_realm')
        self.assertEquals('{"id": "my_realm", "description": "my desc"}', json.dumps(json_resp))

if __name__ == '__main__':
    unittest.main()
