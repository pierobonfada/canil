        CREATE TABLE IF NOT EXISTS admins (
            id INTEGER PRIMARY KEY AUTOINCREMENT, 
            name TEXT NOT NULL, 
            email TEXT NOT NULL UNIQUE, 
            phone TEXT NOT NULL, 
            password TEXT NOT NULL, 
            is_active INTEGER NOT NULL DEFAULT 1,
            is_master INTEGER NOT NULL DEFAULT 0,
            is_first_login INTEGER NOT NULL DEFAULT 1,
            pref_show_inactive INTEGER NOT NULL DEFAULT 0,
            pref_show_others INTEGER NOT NULL DEFAULT 0,
            pref_sort_by TEXT NOT NULL DEFAULT 'updated_desc',
            failed_attempts INTEGER NOT NULL DEFAULT 0,
            is_locked INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS login_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT NOT NULL,
            success INTEGER NOT NULL,
            remote_ip TEXT NOT NULL,
            severity TEXT NOT NULL DEFAULT "INFO",
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS security_warnings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            datetime DATETIME DEFAULT CURRENT_TIMESTAMP,
            msg TEXT NOT NULL,
            remote_ip TEXT NOT NULL,
            endpoint TEXT NOT NULL,
            user_agent TEXT NOT NULL,
            severity TEXT NOT NULL DEFAULT "CRITICAL"
        );

        CREATE TABLE IF NOT EXISTS action_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            admin_id INTEGER NOT NULL,
            action TEXT NOT NULL,
            severity TEXT NOT NULL DEFAULT "INFO",
            animal_id INTEGER,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(admin_id) REFERENCES admins(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS animals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            species TEXT NOT NULL,
            birth_year INTEGER NOT NULL,
            breed TEXT,
            is_vaccinated INTEGER DEFAULT 0,
            is_dewormed INTEGER DEFAULT 0,
            behavior_dogs TEXT,
            behavior_cats TEXT,
            behavior_humans TEXT,
            independence TEXT,
            size TEXT,
            coat_color TEXT,
            coat_length TEXT,
            description TEXT,
            is_active INTEGER DEFAULT 1,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS animal_diseases (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            animal_id INTEGER NOT NULL,
            disease_name TEXT NOT NULL,
            FOREIGN KEY(animal_id) REFERENCES animals(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS animal_photos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            animal_id INTEGER NOT NULL,
            file_path TEXT NOT NULL,
            is_primary INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY(animal_id) REFERENCES animals(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS animal_tutors (
            animal_id INTEGER NOT NULL,
            admin_id INTEGER NOT NULL,
            PRIMARY KEY (animal_id, admin_id),
            FOREIGN KEY(animal_id) REFERENCES animals(id) ON DELETE CASCADE,
            FOREIGN KEY(admin_id) REFERENCES admins(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS site_analytics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            visitor_id TEXT NOT NULL,
            ip_address TEXT,
            user_agent TEXT,
            event_type TEXT NOT NULL,
            path TEXT NOT NULL,
            animal_id INTEGER,
            payload TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
