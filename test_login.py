import requests

# Test if admin dashboard returns admin index.html or site index.html
resp = requests.get("http://localhost:8080/admin/dashboard")
if "Adote seu PET" in resp.text:
    print("Returned PUBLIC SITE")
elif "Acesso ao Painel" in resp.text or "Painel do Canil" in resp.text or "admin" in resp.text.lower():
    print("Returned ADMIN SITE")
else:
    print("Returned SOMETHING ELSE")
    print(resp.text[:100])
