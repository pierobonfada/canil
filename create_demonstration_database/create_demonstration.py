import os
import requests
import urllib.request
import time
import random
import argparse
import sys
import uuid
import json

def login(url, email, password):
    res = requests.post(f"{url}/auth/login", json={"email": email, "password": password})
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

def create_animal(url, token, species, name, photo_path, delay):
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
        res = requests.post(f"{url}/animais", headers=headers, data=data, files=files)
        if res.status_code != 200:
            print(f"Failed to create {name}: {res.text}")
        else:
            print(f"Success: {name}")
        time.sleep(delay)

def generate_traffic(url):
    print("Generating simulated traffic for 20+ users...")
    res = requests.get(f"{url}/public/animais?limit=100")
    if res.status_code != 200:
        print("Failed to fetch animals for traffic generation.")
        return
    animals = res.json()
    if not animals:
        print("Nenhum animal encontrado para gerar tráfego.")
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
        
        def send_event(ev_type, path, animal_id=None, payload=None):
            data = {
                "visitor_id": visitor_id,
                "event_type": ev_type,
                "path": path,
                "animal_id": animal_id,
                "payload": payload
            }
            # Envia o evento para a API pública
            # A data será definida como a atual (CURRENT_TIMESTAMP) pelo servidor
            requests.post(f"{url}/public/analytics", headers=headers, json=data)

        # Step 1: Visit Home
        send_event("page_view", "/public/animais")
        
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
                send_event("search", "/public/animais", payload=payload)

        # Step 3: View some animals
        for _ in range(random.randint(1, 4)):
            a_id = random.choice(animal_ids)
            send_event("page_view", f"/animal/{a_id}", animal_id=a_id)
            
            # Step 4: Maybe send message
            if random.random() > 0.7:
                method = random.choice(["whatsapp", "email"])
                send_event("message_sent", f"/animal/{a_id}", animal_id=a_id, payload={"method": method, "tutor": "Tutor de Demonstração"})

    print("Traffic generated successfully.")

def main():
    parser = argparse.ArgumentParser(description="Script para popular o banco de dados do Canil via API HTTP")
    parser.add_argument("--url", type=str, default="http://127.0.0.1:8000/api", help="URL base da API")
    parser.add_argument("--email", type=str, required=True, help="Email do administrador logado (criado previamente)")
    parser.add_argument("--password", type=str, required=True, help="Senha do administrador logado")
    parser.add_argument("--populate_animals", action="store_true", help="Cria novos dados de demonstração (acrescenta aos existentes).")
    parser.add_argument("--simulate_traffic", action="store_true", help="Gera tráfego simulado de navegação com a data atual.")
    parser.add_argument("--overflow", action="store_true", help="Cria 150 cachorros e 40 gatos reaproveitando as fotos.")
    parser.add_argument("--delay", type=float, default=0.5, help="Tempo de espera em segundos entre a criação de animais (default 0.5)")
    
    args = parser.parse_args()

    if not args.populate_animals and not args.simulate_traffic and not args.overflow:
        parser.print_help()
        return

    # Garante que as fotos baixadas fiquem salvas
    os.makedirs('photos', exist_ok=True)
    
    if args.populate_animals or args.overflow:
        print(f"Logando como {args.email}...")
        token = login(args.url, args.email, args.password)

        dogs = ["Rex", "Totó", "Bolinha", "Lassie", "Snoopy", "Pluto", "Max", "Buddy", "Duke", "Buster"]
        cats = ["Garfield", "Mingau"]

        is_overflow = args.overflow
        num_dogs = 150 if is_overflow else len(dogs)
        num_cats = 40 if is_overflow else len(cats)

        print("Fetching images and creating profiles. This might take a minute...")

        for i in range(num_dogs):
            photo_index = i % len(dogs)
            name = dogs[photo_index] + (f" {i//len(dogs)}" if i >= len(dogs) else "")
            photo_path = f"photos/dog_{photo_index}.jpg"
            if i < len(dogs):
                download_image(f"https://placedog.net/400/400?id={photo_index+1}", photo_path)
            create_animal(args.url, token, "Cachorro", name, photo_path, args.delay)

        for i in range(num_cats):
            photo_index = i % len(cats)
            name = cats[photo_index] + (f" {i//len(cats)}" if i >= len(cats) else "")
            photo_path = f"photos/cat_{photo_index}.jpg"
            if i < len(cats):
                download_image(f"https://placecats.com/400/400?id={photo_index+1}", photo_path)
            create_animal(args.url, token, "Gato", name, photo_path, args.delay)

    if args.simulate_traffic or args.populate_animals or args.overflow:
        generate_traffic(args.url)
        
    print("Done!")

if __name__ == "__main__":
    main()
