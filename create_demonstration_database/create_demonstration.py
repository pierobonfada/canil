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
            'behavior_dogs': random.choice(['Dócil', 'Neutro', 'Agressivo', 'Desconhecido']),
            'behavior_cats': random.choice(['Dócil', 'Neutro', 'Agressivo', 'Desconhecido']),
            'behavior_humans': random.choice(['Dócil', 'Medroso', 'Agressivo', 'Desconhecido']),
            'independence': random.choice(['Independente', 'Dependente']),
            'size': random.choice(['Pequeno', 'Médio', 'Grande']),
            'coat_color': 'Variada',
            'predominant_color': random.choice(['Branco', 'Preto', 'Caramelo', 'Cinza', 'Marrom', 'Tricolor', 'Bicolor', 'Laranja', 'Outra']),
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
    if '--help' in sys.argv or len(sys.argv) < 2:
        print("Uso do script:")
        print("  --populate_animals   Limpa os dados antigos e cria novos dados de demonstração.")
        print("  --simulate_traffic   Gera tráfego simulado de navegação para as estatísticas.")
        print("  --clear              Apenas remove os dados de demonstração (não cria novos).")
        print("  --overflow           Cria 150 cachorros e 40 gatos reaproveitando as fotos.")
        return

    os.makedirs('photos', exist_ok=True)
    
    if '--clear' in sys.argv or '--populate_animals' in sys.argv or '--overflow' in sys.argv:
        cleanup()

    if '--clear' in sys.argv and '--populate_animals' not in sys.argv and '--overflow' not in sys.argv:
        print("Clear mode enabled. Exiting sem criar novos animais.")
        return

    if '--populate_animals' in sys.argv or '--overflow' in sys.argv:
        setup_db_user()
        token = login()

        dogs = ["Rex", "Totó", "Bolinha", "Lassie", "Snoopy", "Pluto", "Max", "Buddy", "Duke", "Buster"]
        cats = ["Garfield", "Mingau"]

        is_overflow = '--overflow' in sys.argv
        num_dogs = 150 if is_overflow else len(dogs)
        num_cats = 40 if is_overflow else len(cats)

        print("Fetching images and creating profiles. This might take a minute...")

        for i in range(num_dogs):
            photo_index = i % len(dogs)
            name = dogs[photo_index] + (f" {i//len(dogs)}" if i >= len(dogs) else "")
            photo_path = f"photos/dog_{photo_index}.jpg"
            if i < len(dogs):
                download_image(f"https://loremflickr.com/400/400/dog?lock={photo_index+10}", photo_path)
            create_animal(token, "Cachorro", name, photo_path)

        for i in range(num_cats):
            photo_index = i % len(cats)
            name = cats[photo_index] + (f" {i//len(cats)}" if i >= len(cats) else "")
            photo_path = f"photos/cat_{photo_index}.jpg"
            if i < len(cats):
                download_image(f"https://loremflickr.com/400/400/cat?lock={photo_index+10}", photo_path)
            create_animal(token, "Gato", name, photo_path)

    if '--simulate_traffic' in sys.argv or '--populate_animals' in sys.argv or '--overflow' in sys.argv:
        generate_traffic()
        
    print("Done!")

import uuid
from datetime import datetime, timedelta

def generate_traffic():
    print("Generating simulated traffic for 20+ users...")
    res = requests.get(f"{API_URL}/public/animais?limit=100")
    if res.status_code != 200:
        print("Failed to fetch animals for traffic generation.")
        return
    animals = res.json()
    if not animals:
        return

    animal_ids = [a['id'] for a in animals]
    animal_species = list(set([a['species'] for a in animals]))
    animal_colors = ["Preto", "Branco", "Caramelo", "Mesclado", "Frajola"]
    animal_sizes = ["Pequeno", "Médio", "Grande"]

    user_agents = [
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 16_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.5 Mobile/15E148 Safari/604.1",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.5 Safari/605.1.15",
        "Mozilla/5.0 (Linux; Android 13; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Mobile Safari/537.36"
    ]

    for _ in range(random.randint(20, 30)):
        visitor_id = "v_" + str(uuid.uuid4())
        ua = random.choice(user_agents)
        ip = f"{random.randint(100,200)}.{random.randint(10,250)}.{random.randint(10,250)}.{random.randint(1,250)}"
        headers = {"User-Agent": ua, "X-Forwarded-For": ip}
        
        # Base time for this user (up to 30 days ago)
        base_time = datetime.now() - timedelta(days=random.randint(0, 30), hours=random.randint(0, 23))
        
        def send_event(ev_type, path, animal_id=None, payload=None, time_offset_sec=0):
            # SQLite manipulation to insert retroactively (API uses current time)
            # Since our API uses CURRENT_TIMESTAMP, we will insert directly via sqlite
            conn = sqlite3.connect(DB_PATH)
            cursor = conn.cursor()
            ev_time = (base_time + timedelta(seconds=time_offset_sec)).strftime('%Y-%m-%d %H:%M:%S')
            cursor.execute('''
                INSERT INTO site_analytics (visitor_id, ip_address, user_agent, event_type, path, animal_id, payload, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ''', (visitor_id, ip, ua, ev_type, path, animal_id, payload, ev_time))
            conn.commit()
            conn.close()

        # Step 1: Visit Home
        send_event("page_view", "/public/animais", time_offset_sec=0)
        
        offset = random.randint(10, 60)
        # Step 2: Search filters
        for _ in range(random.randint(0, 3)):
            payload = {}
            if random.random() > 0.5:
                payload["species"] = random.choice(animal_species)
            if random.random() > 0.5:
                payload["predominant_color"] = random.choice(animal_colors)
            if random.random() > 0.5:
                payload["size"] = random.choice(animal_sizes)
            if payload:
                import json
                send_event("search", "/public/animais", payload=json.dumps(payload), time_offset_sec=offset)
                offset += random.randint(5, 20)

        # Step 3: View some animals
        for _ in range(random.randint(1, 4)):
            a_id = random.choice(animal_ids)
            send_event("page_view", f"/animal/{a_id}", animal_id=a_id, time_offset_sec=offset)
            offset += random.randint(10, 120)
            
            # Step 4: Maybe send message
            if random.random() > 0.7:
                method = random.choice(["whatsapp", "email"])
                import json
                send_event("message_sent", f"/animal/{a_id}", animal_id=a_id, payload=json.dumps({"method": method, "tutor": "Tutor de Demonstração"}), time_offset_sec=offset+10)

    print("Traffic generated successfully.")

if __name__ == "__main__":
    main()
