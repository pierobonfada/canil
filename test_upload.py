import requests

url = "http://localhost:8080/api/animais"
headers = {"Authorization": "Bearer test1234"} # not valid token, but maybe we get 401 instead of 413
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

try:
    response = requests.post(url, headers=headers, data=data, files=files)
    print(f"Status: {response.status_code}")
    print(response.text)
except Exception as e:
    print(f"Error: {e}")
