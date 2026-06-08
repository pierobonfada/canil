import sqlite3
c=sqlite3.connect('/home/p/canil/api-canil/canil.db')
print(c.execute('SELECT sql FROM sqlite_master WHERE type="table" AND name="action_logs"').fetchone()[0])
