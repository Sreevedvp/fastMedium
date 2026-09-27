#!/usr/bin/env bash
# One command for the Dioxus browser app and standalone Tokio API.
set -euo pipefail
cd "$(dirname "$0")"
exec python3 -u - "$@" <<'PY'
import os, signal, socket, subprocess, sys, time, urllib.request, shutil
from pathlib import Path
root = Path.cwd()
web_port = int(os.environ.get('WEB_PORT', '8080'))
api_port = int(os.environ.get('API_PORT', '8082'))
children = []
for command in ('cargo', 'dx'):
    if not shutil.which(command):
        sys.exit(f'Missing {command}. See README.md for setup.')
if web_port == api_port:
    sys.exit('WEB_PORT and API_PORT must be different.')
for port in (web_port, api_port):
    with socket.socket() as probe:
        probe.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        try: probe.bind(('127.0.0.1', port))
        except OSError: sys.exit(f'Port {port} is already in use. Stop the existing server or set WEB_PORT/API_PORT.')
env = os.environ.copy()
env.update(WEB_PORT=str(web_port), API_PORT=str(api_port), FASTMEDIUM_API_PORT=str(api_port))
# cargo/dx and the API all run at the project root, so relative SQLite paths are stable.
# The API loads .env itself; exported environment values take precedence.
def start(command):
    child = subprocess.Popen(command, cwd=root, env=env, start_new_session=True)
    children.append(child)
    return child

def stop(*_):
    raise KeyboardInterrupt
signal.signal(signal.SIGTERM, stop)
signal.signal(signal.SIGINT, stop)
exit_code = 0
try:
    print('Building the Tokio API…', flush=True)
    build = start(['cargo', 'build', '--bin', 'fast-medium-api', '--no-default-features', '--features', 'server'])
    if build.wait() != 0: sys.exit(build.returncode)
    target = Path(env.get('CARGO_TARGET_DIR', root / 'target'))
    api = start([str(target.resolve() / 'debug' / 'fast-medium-api')])
    deadline = time.monotonic() + 30
    while True:
        if api.poll() is not None: sys.exit('The API exited before it became ready.')
        try:
            with urllib.request.urlopen(f'http://127.0.0.1:{api_port}/api/health', timeout=1) as response:
                if response.status == 200: break
        except (OSError, TimeoutError): pass
        if time.monotonic() >= deadline: sys.exit('API readiness timed out. Check DATABASE_URL and the startup logs.')
        time.sleep(.2)
    print(f'API ready: http://127.0.0.1:{api_port}/api/health', flush=True)
    print(f'Starting browser app: http://127.0.0.1:{web_port} (first build may take a moment)', flush=True)
    web = start(['dx', 'serve', '--platform', 'web', '--bin', 'fast-medium', '--fullstack', 'false', '--port', str(web_port), '--addr', '127.0.0.1', '--open', 'false', '--interactive', 'false'])
    while api.poll() is None and web.poll() is None: time.sleep(.5)
    exit_code = (api.poll() if api.poll() is not None else web.poll()) or 1
    print('A service exited; stopping the other service.', flush=True)
except KeyboardInterrupt:
    print('\nStopping fastMedium…', flush=True)
finally:
    signal.signal(signal.SIGTERM, signal.SIG_IGN)
    for child in reversed(children):
        try: os.killpg(child.pid, signal.SIGTERM)
        except ProcessLookupError: pass
    for child in reversed(children):
        try: child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            try: os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError: pass
            child.wait()
sys.exit(exit_code)
PY
