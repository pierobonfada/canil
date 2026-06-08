import requests

url_login = "http://localhost:8000/api/auth/login"
resp = requests.post(url_login, json={"email": "master@master.master", "password": "wrong"})
print(f"Login failed: {resp.status_code}")
print(resp.text)
