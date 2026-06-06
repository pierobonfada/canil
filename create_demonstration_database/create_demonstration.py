import os
import sqlite3
import requests
import urllib.request
import time
import random

API_URL = "http://127.0.0.1:8000/api"
DB_PATH = "../api-canil/canil.db"
API_ROOT_DIR = "../api-canil"
EMAIL = "demonstracao@demonstracao.demonstracao"
PASSWORD = "Demonstracao123!"
PASSWORD_HASH = "$2b$12$eIMNjxMVJ980wZYiDm7IdOR9moXuUve68mIurLi0ahcxkUJ92os4W"

def cleanup():
    print("Starting cleanup of previous demonstration data...")
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    cursor.execute("SELECT id FROM admins WHERE email = ?", (EMAIL,))
    user = cursor.fetchone()
    if not user:
        print("No demonstracao user found. Skipping cleanup.")
        conn.close()
        return

    admin_id = user[0]
    cursor.execute("SELECT animal_id FROM animal_tutors WHERE admin_id = ?", (admin_id,))
    animals = cursor.fetchall()
    
    for (animal_id,) in animals:
        # Delete photos physically
        cursor.execute("SELECT file_path FROM animal_photos WHERE animal_id = ?", (animal_id,))
        photos = cursor.fetchall()
        for (file_path,) in photos:
            full_path = os.path.join(API_ROOT_DIR, file_path)
            if os.path.exists(full_path):
                os.remove(full_path)
                print(f"Removed photo: {full_path}")
        
        # Hard delete from database
        cursor.execute("DELETE FROM animal_photos WHERE animal_id = ?", (animal_id,))
        cursor.execute("DELETE FROM animal_diseases WHERE animal_id = ?", (animal_id,))
        cursor.execute("DELETE FROM animal_tutors WHERE animal_id = ?", (animal_id,))
        cursor.execute("DELETE FROM animals WHERE id = ?", (animal_id,))
        print(f"Hard deleted animal ID: {animal_id}")

    conn.commit()
    conn.close()
    print("Cleanup complete.")

def setup_db_user():
    conn = sqlite3.connect(DB_PATH)
    cursor = conn.cursor()
    cursor.execute("SELECT id FROM admins WHERE email = ?", (EMAIL,))
    user = cursor.fetchone()
    if not user:
        cursor.execute('''
            INSERT INTO admins (name, email, phone, password, is_master, is_first_login, is_active, failed_attempts, is_locked)
            VALUES (?, ?, ?, ?, 0, 0, 1, 0, 0)
        ''', ("Tutor de Demonstração", EMAIL, "0000000000", PASSWORD_HASH))
        conn.commit()
        print(f"Created user {EMAIL}")
    else:
        print(f"User {EMAIL} already exists.")
    conn.close()

def login():
    res = requests.post(f"{API_URL}/auth/login", json={"email": EMAIL, "password": PASSWORD})
    if res.status_code == 200:
        return res.json()["token"]
    raise Exception(f"Login failed: {res.text}")

def download_image(url, filename):
    if not os.path.exists(filename):
        print(f"Downloading {filename}...")
        req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
        with urllib.request.urlopen(req) as response, open(filename, 'wb') as out_file:
            out_file.write(response.read())
        time.sleep(0.5)

def create_animal(token, species, name, photo_path):
    print(f"Creating {species}: {name}")
    with open(photo_path, 'rb') as f:
        files = {'photo': (os.path.basename(photo_path), f, 'image/jpeg')}
        data = {
            'name': name,
            'species': species,
            'birth_year': str(random.randint(2015, 2024)),
            'breed': 'SRD' if random.random() > 0.5 else ('Labrador' if species == 'Cachorro' else 'Persa'),
            'is_vaccinated': 'true',
            'is_dewormed': 'true',
            'behavior_dogs': 'Sociável',
            'behavior_humans': 'Amigável',
            'independence': 'Moderada',
            'size': 'Médio',
            'coat_color': 'Variada',
            'coat_length': 'Curto',
            'description': f"Este é {name}, um ótimo {species.lower()} aguardando muito carinho e uma nova casa!"
        }
        headers = {'Authorization': f'Bearer {token}'}
        res = requests.post(f"{API_URL}/animais", headers=headers, data=data, files=files)
        if res.status_code != 200:
            print(f"Failed to create {name}: {res.text}")
        else:
            print(f"Success: {name}")

import sys

def main():
    if len(sys.argv) < 2 or (sys.argv[1] not in ['--create', '--clear']):
        print("Uso do script:")
        print("  --create   Limpa os dados antigos e cria novos dados de demonstração.")
        print("  --clear    Apenas remove os dados de demonstração (não cria novos).")
        return

    os.makedirs('photos', exist_ok=True)
    cleanup()

    if '--clear' in sys.argv:
        print("Clear mode enabled. Exiting sem criar novos animais.")
        return

    setup_db_user()
    token = login()

    dogs = ["Rex", "Totó", "Bolinha", "Lassie", "Snoopy", "Pluto", "Max", "Buddy", "Duke", "Buster"]
    cats = ["Garfield", "Mingau"]

    print("Fetching images and creating profiles. This might take a minute...")

    for i, name in enumerate(dogs):
        photo_path = f"photos/dog_{i}.jpg"
        download_image(f"https://loremflickr.com/400/400/dog?lock={i+10}", photo_path)
        create_animal(token, "Cachorro", name, photo_path)

    for i, name in enumerate(cats):
        photo_path = f"photos/cat_{i}.jpg"
        download_image(f"https://loremflickr.com/400/400/cat?lock={i+10}", photo_path)
        create_animal(token, "Gato", name, photo_path)

    print("Done!")

if __name__ == "__main__":
    main()
