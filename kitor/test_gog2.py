import sqlite3
import urllib.request
import json
import ssl
import os

ctx = ssl.create_default_context()
ctx.check_hostname = False
ctx.verify_mode = ssl.CERT_NONE

db_path = os.path.expanduser('~/Library/Application Support/haggai/kitor.db')
conn = sqlite3.connect(db_path)
c = conn.cursor()
c.execute("SELECT access_token FROM gog_auth")
row = c.fetchone()
conn.close()

if not row:
    print("No token")
    exit(1)

import base64
token = base64.b64decode(row[0]).decode('utf-8')

url = f"https://www.gog.com/downloads/the_secret_of_monkey_island_special_edition/en1installer0?access_token={token}"
print(url[:50] + "...")
req = urllib.request.Request(url)
try:
    resp = urllib.request.urlopen(req, context=ctx)
    print("URL:", resp.url)
    print("Status:", resp.status)
except Exception as e:
    print(e)
