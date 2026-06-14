#!/usr/bin/env python3
"""Register test users on a local Synapse instance via admin API."""
import json
import urllib.request
import hmac
import hashlib

HS = 'http://localhost:8008'
SECRET = b's3cret!'
USERS = [('alice', 'pass123'), ('bob', 'pass456')]

for username, password in USERS:
    req = urllib.request.Request(f'{HS}/_synapse/admin/v1/register')
    resp = json.loads(urllib.request.urlopen(req).read())
    nonce = resp['nonce']

    mac = hmac.new(SECRET, digestmod=hashlib.sha1)
    mac.update(nonce.encode('utf8'))
    mac.update(b'\x00')
    mac.update(username.encode('utf8'))
    mac.update(b'\x00')
    mac.update(password.encode('utf8'))
    mac.update(b'\x00')
    mac.update(b'notadmin')

    body = json.dumps({
        'nonce': nonce,
        'username': username,
        'password': password,
        'admin': False,
        'mac': mac.hexdigest(),
    }).encode()
    req = urllib.request.Request(
        f'{HS}/_synapse/admin/v1/register',
        data=body,
        headers={'Content-Type': 'application/json'},
    )
    resp = json.loads(urllib.request.urlopen(req).read())
    print(f'Registered {username}: {resp.get("user_id", "ok")}')
