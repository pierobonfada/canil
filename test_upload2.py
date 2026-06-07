import requests

url_login = "http://localhost:8080/api/auth/login"
resp = requests.post(url_login, json={"email": "master@master.master", "password": "senha"})
if resp.status_code != 200:
    resp = requests.post(url_login, json={"email": "master@master.master", "password": "teste01"})
if resp.status_code != 200:
    print(f"Login failed: {resp.text}")
    exit(1)
token = resp.json()["token"]

url = "http://localhost:8080/api/animais"
headers = {"Authorization": f"Bearer {token}"}
files = [
    ('photo', ('test1.jpg', b'0' * 5 * 1024 * 1024, 'image/jpeg')),
    ('photo', ('test2.jpg', b'0' * 5 * 1024 * 1024, 'image/jpeg'))
]
data = {
    'name': 'Test',
    'species': 'Gato',
    'birth_year': '2020',
    'breed': 'SRD',
    'is_vaccinated': 'true',
    'is_dewormed': 'true',
    'behavior_dogs': 'Neutro',
    'behavior_cats': 'Neutro',
    'behavior_humans': 'Dócil',
    'independence': 'Independente',
    'size': 'Pequeno',
    'coat_color': 'Preto',
    'predominant_color': 'Preto',
    'coat_length': 'Curto',
    'description': 'Test description'
}

print("Uploading...")
try:
    response = requests.post(url, headers=headers, data=data, files=files)
    print(f"Status: {response.status_code}")
    print(response.text)
except Exception as e:
    print(f"Error: {e}")
