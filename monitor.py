import subprocess
import time

def get_mem():
    res = subprocess.run(["docker", "stats", "--no-stream", "--format", "{{.MemUsage}}", "canil_test"], capture_output=True, text=True)
    val = res.stdout.split('/')[0].strip()
    if 'MiB' in val:
        return float(val.replace('MiB', ''))
    elif 'GiB' in val:
        return float(val.replace('GiB', '')) * 1024
    elif 'kB' in val:
        return float(val.replace('kB', '')) / 1024
    elif 'B' in val:
        return float(val.replace('B', '')) / 1024 / 1024
    return 0

max_mem = 0
for _ in range(300):
    try:
        mem = get_mem()
        if mem > max_mem:
            max_mem = mem
    except:
        pass
    time.sleep(0.5)

print(f"Peak Memory: {max_mem} MiB")
