import unittest
import requests
import json

class BaseTest(unittest.TestCase):
    # default setup method
    def setUp(self):
        self._realm = {"id": "default"}
        self._principal = {"id": "00000000-0000-0000-0000-000000000000"}
        self._base_url = 'http://localhost:8000'

    def post(self, path, data):
        headers = {'Content-Type': 'application/json', 'X-Principal': self._principal["id"], 'X-Realm': self._realm["id"]}
        r = requests.post(self._base_url + path, data=json.dumps(data), headers=headers)
        #print("POST %s RC: %d\n" % (path, r.status_code))
        if r.status_code != 200:
            raise Exception(r.text)
        #
        try:
            #print("response from %s: %s\n" % (path, r.text))
            return json.loads(r.text)
        except:
            return r.text

    def put(self, path, data):
        headers = {'Content-Type': 'application/json', 'X-Principal': self._principal["id"], 'X-Realm': self._realm["id"]}
        r = requests.put(self._base_url + path, data=json.dumps(data), headers=headers)
        #print("PUT %s RC: %d\n" % (path, r.status_code))
        if r.status_code != 200:
            raise Exception(r.text)
        try:
            return json.loads(r.text)
        except:
            return r.text

    def delete(self, path):
        headers = {'Content-Type': 'application/json', 'X-Principal': self._principal["id"], 'X-Realm': self._realm["id"]}
        r = requests.delete(self._base_url + path, headers=headers)
        #print("DELETE %s RC: %d\n" % (path, r.status_code))
        if r.status_code != 200:
            raise Exception(r.text)
        try:
            return json.loads(r.text)
        except:
            return r.text

    def get(self, path):
        headers = {'Content-Type': 'application/json', 'X-Principal': self._principal["id"], 'X-Realm': self._realm["id"]}
        r = requests.get(self._base_url + path, headers=headers)
        #print("GET %s RC: %d\n" % (path, r.status_code))
        if r.status_code != 200:
            raise Exception(r.text)
        try:
            return json.loads(r.text)
        except:
            return r.text
