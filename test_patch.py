import requests

url_login = "http://localhost:8000/api/auth/login"
resp = requests.post(url_login, json={"email": "master@master.master", "password": "teste01"})
if resp.status_code != 200:
    print(f"Login failed: {resp.text}")
    exit(1)
token = resp.json()["token"]

url = "http://localhost:8000/api/animais/1/status"
headers = {"Authorization": f"Bearer {token}"}
resp = requests.patch(url, headers=headers, json={"is_active": False})
print(f"Patch status: {resp.status_code}")
print(resp.text)
