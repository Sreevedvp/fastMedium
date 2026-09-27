#!/usr/bin/env python3
"""Exercise the real Tokio HTTP API using a disposable database; no external services."""
import http.cookiejar
import hashlib
import uuid
from concurrent.futures import ThreadPoolExecutor
import json
import os
from pathlib import Path
import socket
import sqlite3
import subprocess
import tempfile
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get('CARGO_TARGET_DIR', ROOT / 'target')).resolve() / 'debug/fast-medium-api'
if not BINARY.exists():
    raise SystemExit('Build first: cargo build --bin fast-medium-api --no-default-features --features server')
with socket.socket() as s:
    s.bind(('127.0.0.1', 0))
    PORT = s.getsockname()[1]
BASE = f'http://127.0.0.1:{PORT}'

def client():
    return urllib.request.build_opener(urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()))

def call(c, method, path, body=None, expected=200, headers=None):
    data = None if body is None else json.dumps(body).encode()
    req = urllib.request.Request(BASE + path, data=data, method=method,
                                 headers={'Content-Type': 'application/json', **(headers or {})})
    try: response = c.open(req, timeout=10)
    except urllib.error.HTTPError as e: response = e
    raw = response.read()
    assert response.status == expected, (method, path, response.status, raw.decode())
    return json.loads(raw) if raw else None, response.headers

with tempfile.TemporaryDirectory(prefix='fastmedium-api-check-') as tmp:
    env = {**os.environ, 'API_PORT': str(PORT), 'WEB_PORT': '8080',
           'DATABASE_URL': f'sqlite://{tmp}/check.db',
           'API_HOST': '127.0.0.1', 'ALLOWED_ORIGINS': 'http://127.0.0.1:8080,http://localhost:8080', 'COOKIE_SECURE': 'false', 'AUTH_ATTEMPTS_PER_MINUTE': '30',
           'SMTP_USER': 'your-email', 'SMTP_PASS': 'your-app-password'}
    log = open(Path(tmp) / 'api.log', 'w+')
    process = None
    def start():
        global process
        process = subprocess.Popen([str(BINARY)], env=env, cwd=tmp, stdout=log, stderr=log)
        for _ in range(100):
            if process.poll() is not None:
                log.seek(0); raise AssertionError(log.read())
            try:
                call(client(), 'GET', '/api/health')
                return
            except OSError: time.sleep(.1)
        raise AssertionError('API failed to start')
    def stop():
        if process and process.poll() is None:
            process.terminate()
            try: process.wait(timeout=5)
            except subprocess.TimeoutExpired: process.kill(); process.wait()
    legacy_token = str(uuid.uuid4())
    # Start from the previous schema to verify the additive upgrade path.
    with sqlite3.connect(Path(tmp) / 'check.db') as db:
        db.executescript((ROOT / 'migrations/001_initial.sql').read_text())
        db.execute("INSERT INTO users(id,username,display_name,email,password_hash) VALUES ('legacy','legacy_writer','Legacy writer','legacy@example.invalid','not-used')")
        db.execute("INSERT INTO posts(id,author_id,title,slug) VALUES ('legacy-post','legacy','Existing draft','existing-draft')")
        db.execute("INSERT INTO sessions(id,user_id,token,expires_at) VALUES ('legacy-session','legacy',?,'2099-01-01 00:00:00')",(legacy_token,))
    try:
        start()
        a, b, public = client(), client(), client()
        legacy, _ = call(public, 'GET', '/api/auth/me', headers={'Cookie': 'fm_session=' + legacy_token})
        assert legacy['username'] == 'legacy_writer', 'Legacy session was not preserved'
        with sqlite3.connect(Path(tmp) / 'check.db') as db:
            assert db.execute('SELECT COUNT(*) FROM fm_migrations').fetchone()[0] == 3
            assert db.execute("SELECT token FROM sessions WHERE id='legacy-session'").fetchone()[0] == hashlib.sha256(legacy_token.encode()).hexdigest()
        _, response_headers = call(public, 'GET', '/api/health')
        assert response_headers['Cache-Control'] == 'no-store' and response_headers['X-Content-Type-Options'] == 'nosniff'
        assert uuid.UUID(response_headers['X-Request-Id'])
        call(public, 'POST', '/api/auth/register', {}, expected=403, headers={'Origin': 'https://untrusted.example'})
        call(public, 'POST', '/api/auth/login', {}, expected=422)
        call(public, 'GET', '/api/no-such-route', expected=404)
        call(public, 'GET', '/api/posts?page=bad', expected=400)

        account = {'username': 'regression_writer', 'display_name': 'Regression Writer',
                   'email': 'regression@example.invalid', 'password': 'Local-test-only-123!'}
        user, headers = call(a, 'POST', '/api/auth/register', account)
        assert 'HttpOnly' in headers['Set-Cookie'] and 'SameSite=Strict' in headers['Set-Cookie']
        assert 'password_hash' not in user
        me, _ = call(a, 'GET', '/api/auth/me'); assert me['id'] == user['id']
        call(public, 'GET', '/api/posts/mine', expected=401)
        call(public, 'POST', '/api/auth/login', {'email': account['email'], 'password': 'wrong'}, expected=401)
        call(a, 'POST', '/api/auth/logout')
        me, _ = call(a, 'GET', '/api/auth/me'); assert me is None
        call(a, 'GET', '/api/posts/mine', expected=401)
        call(a, 'POST', '/api/auth/login', {'email': '  REGRESSION@EXAMPLE.INVALID  ', 'password': account['password']})
        call(b, 'POST', '/api/auth/register', {**account, 'username': 'another_writer', 'email': 'another@example.invalid'})
        draft, _ = call(a, 'POST', '/api/posts', {'title': 'A regression story', 'excerpt': 'A test excerpt', 'content': '## Thought\n\nA new idea.'})
        pid = draft['id']; slug = draft['slug']
        # Expertise tags: normalized, deduplicated, owner-only, public and persisted.
        skills, _ = call(a, 'PUT', '/api/me/skills', {'skills': [' Rust ', 'rust', 'Product   design']})
        assert skills == ['Rust', 'Product design']
        call(public, 'PUT', '/api/me/skills', {'skills': ['unauthorized']}, expected=401)
        call(a, 'PUT', '/api/me/skills', {'skills': ['x'*41]}, expected=400)
        call(a, 'PUT', '/api/me/skills', {'skills': ['']}, expected=400)
        call(a, 'PUT', '/api/me/skills', {'skills': [str(n) for n in range(13)]}, expected=400)
        call(b, 'PUT', '/api/me/skills', {'skills': ['Research'], 'user_id': user['id']})
        skills, _ = call(public, 'GET', '/api/profiles/' + account['username'] + '/skills'); assert skills == ['Rust', 'Product design']
        skills, _ = call(a, 'PUT', '/api/me/skills', {'skills': ['Rust']}); assert skills == ['Rust']
        call(public, 'POST', '/api/publications', {'name': 'Not allowed', 'description': ''}, expected=401)
        call(a, 'POST', '/api/publications', {'name': 'x', 'description': ''}, expected=400)
        publication, _ = call(a, 'POST', '/api/publications', {'name': 'Regression collection', 'description': 'A test publication.'})
        pubid = publication['id']; pubslug = publication['slug']
        call(a, 'POST', f'/api/publications/{pubid}/posts/{pid}')
        detail, _ = call(a, 'GET', '/api/publications/by-slug/' + pubslug); assert detail['can_manage'] and len(detail['posts']) == 1
        detail, _ = call(public, 'GET', '/api/publications/by-slug/' + pubslug); assert not detail['can_manage'] and detail['posts'] == [] and detail['publication']['story_count'] == 0
        call(b, 'PUT', f'/api/publications/{pubid}', {'name':'Stolen collection','description':''}, expected=403)
        call(b, 'DELETE', f'/api/publications/{pubid}', expected=403)
        call(b, 'POST', f'/api/publications/{pubid}/posts/{pid}', expected=403)
        call(b, 'DELETE', f'/api/publications/{pubid}/posts/{pid}', expected=403)
        foreign_post, _ = call(b, 'POST', '/api/posts', {'title':'Other writer draft','excerpt':'','content':'Private'})
        call(a, 'POST', f"/api/publications/{pubid}/posts/{foreign_post['id']}", expected=403)
        call(a, 'PUT', f'/api/publications/{pubid}', {'name':'Edited collection','description':'Updated description'})
        second, _ = call(a, 'POST', '/api/publications', {'name':'Regression collection','description':''})
        assert second['slug'] != pubslug
        call(a, 'POST', f"/api/publications/{second['id']}/posts/{pid}")
        detail, _ = call(a, 'GET', '/api/publications/by-slug/' + pubslug); assert detail['posts'] == []
        call(a, 'POST', f'/api/publications/{pubid}/posts/{pid}')
        call(a, 'DELETE', f"/api/publications/{second['id']}")
        item, _ = call(public, 'GET', '/api/posts/by-slug/' + slug); assert item is None
        call(b, 'GET', '/api/posts/' + pid, expected=403)
        call(b, 'PUT', '/api/posts/' + pid, {'title': 'Not mine', 'excerpt': '', 'content': ''}, expected=403)
        call(b, 'POST', '/api/posts/' + pid + '/publish', expected=403)
        call(b, 'DELETE', '/api/posts/' + pid, expected=403)
        call(a, 'PUT', '/api/posts/' + pid, {'title': 'An edited story', 'excerpt': 'Updated excerpt', 'content': '## Thought\n\nAn updated idea.', 'version': draft['version']})
        # Reject stale or unversioned writes without changing the winning content.
        call(a, 'PUT', '/api/posts/' + pid, {'title':'stale','excerpt':'','content':'lost write','version':draft['version']}, expected=409)
        call(a, 'PUT', '/api/posts/' + pid, {'title':'missing version','excerpt':'','content':''}, expected=428)
        call(a, 'GET', '/api/posts/does-not-exist', expected=404)
        call(a, 'POST', '/api/posts/' + pid + '/publish')
        published, _ = call(public, 'GET', '/api/posts/by-slug/' + slug)
        call(a, 'POST', '/api/posts/' + pid + '/publish')
        repeated, _ = call(public, 'GET', '/api/posts/by-slug/' + slug)
        assert repeated['version'] == published['version'] and repeated['published_at'] == published['published_at']

        feed, _ = call(public, 'GET', '/api/posts'); assert any(p['id'] == pid for p in feed)
        item, _ = call(public, 'GET', '/api/posts/by-slug/' + slug); assert item['title'] == 'An edited story' and '<h2>Thought</h2>' in item['content_html']
        profile, _ = call(public, 'GET', '/api/profiles/' + account['username']); assert not profile['email']
        _, headers = call(a, 'GET', '/api/auth/me', headers={'Origin': 'http://127.0.0.1:8080'})
        assert headers['Access-Control-Allow-Origin'] == 'http://127.0.0.1:8080'
        assert headers['Access-Control-Allow-Credentials'] == 'true'
        _, headers = call(a, 'GET', '/api/auth/me', headers={'Origin': 'https://untrusted.example'})
        assert headers.get('Access-Control-Allow-Origin') is None
        detail, _ = call(public, 'GET', '/api/publications/by-slug/' + pubslug)
        assert detail['publication']['name'] == 'Edited collection' and detail['publication']['story_count'] == 1 and detail['posts'][0]['id'] == pid
        stop(); start()
        with sqlite3.connect(Path(tmp) / 'check.db') as db:
            assert db.execute("SELECT title FROM posts WHERE id='legacy-post'").fetchone()[0] == 'Existing draft'
        skills, _ = call(public, 'GET', '/api/profiles/' + account['username'] + '/skills'); assert skills == ['Rust']
        detail, _ = call(a, 'GET', '/api/publications/by-slug/' + pubslug); assert detail['posts'][0]['id'] == pid
        call(a, 'DELETE', f'/api/publications/{pubid}/posts/{pid}')
        detail, _ = call(public, 'GET', '/api/publications/by-slug/' + pubslug); assert detail['posts'] == []
        call(a, 'POST', f'/api/publications/{pubid}/posts/{pid}')
        call(a, 'DELETE', f'/api/publications/{pubid}')
        call(public, 'GET', '/api/publications/by-slug/' + pubslug, expected=404)
        remaining, _ = call(a, 'GET', '/api/posts/' + pid); assert remaining['id'] == pid
        me, _ = call(a, 'GET', '/api/auth/me'); assert me['id'] == user['id'], 'Session did not survive API restart'
        item, _ = call(public, 'GET', '/api/posts/by-slug/' + slug); assert item['id'] == pid
        # Validation rejects oversized input without rendering or storing it.
        call(a, 'POST', '/api/posts', {'title':'x'*201,'excerpt':'','content':''}, expected=400)
        call(a, 'POST', '/api/posts', {'title':'Valid','excerpt':'x'*501,'content':''}, expected=400)
        call(a, 'POST', '/api/posts', {'title':'Valid','excerpt':'','content':'x'*250001}, expected=400)
        call(a, 'POST', '/api/posts', {'title':'Valid','excerpt':'','content':'x'*1100000}, expected=413)
        call(a, 'POST', '/api/auth/register', {**account,'password':'x'*1025}, expected=400)
        call(a, 'POST', '/api/auth/register', account, expected=409)
        malicious = '# Safe heading\n\n<script>alert(1)</script>\n\n<img src=x onerror=alert(1)>\n\n[unsafe](javascript:alert%281%29) ![bad](data:text/html,test)'
        testpost, _ = call(a, 'POST', '/api/posts', {'title':'Security fixture','excerpt':'','content':malicious})
        assert '<script>' not in testpost['content_html'] and '<img src=x' not in testpost['content_html']
        assert 'href="javascript:' not in testpost['content_html'] and 'src="data:' not in testpost['content_html']
        assert '<h1>Safe heading</h1>' in testpost['content_html']
        # An explicitly foreign origin must not mutate an authenticated session.
        call(a, 'DELETE', '/api/posts/' + testpost['id'], expected=403, headers={'Origin':'https://untrusted.example'})
        call(a, 'GET', '/api/posts/' + testpost['id'])
        call(a, 'DELETE', '/api/posts/' + testpost['id'])
        # Parallel same-title creates get distinct slugs, and exactly one versioned update wins.
        cookies = next(h.cookiejar for h in a.handlers if isinstance(h,urllib.request.HTTPCookieProcessor))
        cookie = '; '.join(c.name+'='+c.value for c in cookies)
        def parallel_create(_):
            return call(client(),'POST','/api/posts',{'title':'Same concurrent title','excerpt':'','content':'Draft'}, headers={'Cookie':cookie})[0]
        with ThreadPoolExecutor(max_workers=6) as workers:
            concurrent = list(workers.map(parallel_create,range(6)))
        assert len({item['slug'] for item in concurrent}) == 6
        concurrent_post = concurrent[0]
        def parallel_update(index):
            req = urllib.request.Request(BASE+'/api/posts/'+concurrent_post['id'], method='PUT', headers={'Content-Type':'application/json','Cookie':cookie}, data=json.dumps({'title':f'Writer {index}','excerpt':'','content':f'Version from {index}','version':concurrent_post['version']}).encode())
            try: response = urllib.request.urlopen(req,timeout=10)
            except urllib.error.HTTPError as e: response=e
            return response.status
        with ThreadPoolExecutor(max_workers=2) as workers:
            assert sorted(workers.map(parallel_update,range(2))) == [200,409]
        for item in concurrent: call(a,'DELETE','/api/posts/'+item['id'])
        emoji, _ = call(a,'POST','/api/posts',{'title':'✨✨','excerpt':'','content':''})
        assert emoji['slug']; call(a,'DELETE','/api/posts/'+emoji['id'])
        # Search works outside the first page and does not treat wildcards as user intent.
        with sqlite3.connect(Path(tmp)/'check.db') as db:
            for index in range(25):
                db.execute("INSERT INTO posts(id,author_id,title,slug,content,status,published_at) VALUES (?,?,?,?,?,'published','2020-01-01 00:00:00')",(f'search-{index}',user['id'],f'Older story {index}',f'search-{index}','needle-unique' if index == 24 else 'Ordinary content'))
        feed, _ = call(public,'GET','/api/posts?q=needle-unique'); assert len(feed)==1 and feed[0]['id']=='search-24'
        feed, _ = call(public,'GET','/api/posts?q=%25'); assert feed==[]
        feed, _ = call(public,'GET','/api/posts?page=4294967295&per_page=4294967295'); assert feed==[]
        feed, _ = call(public,'GET','/api/posts?per_page=0'); assert len(feed)==1
        first, _ = call(public,'GET','/api/posts?page=1&per_page=5')
        second_page, _ = call(public,'GET','/api/posts?page=2&per_page=5')
        assert len(first)==len(second_page)==5 and not {p['id'] for p in first}.intersection(p['id'] for p in second_page)
        with sqlite3.connect(Path(tmp)/'check.db') as db:
            db.execute("DELETE FROM posts WHERE id LIKE 'search-%'")
            db.execute('ALTER TABLE posts RENAME TO unavailable_posts')
        body, _ = call(public,'GET','/api/posts',expected=500)
        assert 'no such table' not in body['error'] and 'SELECT' not in body['error']
        with sqlite3.connect(Path(tmp)/'check.db') as db: db.execute('ALTER TABLE unavailable_posts RENAME TO posts')
        # A live WAL backup restores accounts, stories, and sessions in a separately opened database.
        backup = Path(tmp)/'verified-backup.db'
        subprocess.run(['python3',str(ROOT/'scripts/backup_db.py'),str(Path(tmp)/'check.db'),str(backup)],check=True,capture_output=True,text=True)
        assert subprocess.run(['python3',str(ROOT/'scripts/backup_db.py'),str(Path(tmp)/'check.db'),str(backup)],capture_output=True).returncode != 0
        stop()
        original_url = env['DATABASE_URL']; env['DATABASE_URL'] = 'sqlite://'+str(backup)
        start()
        restored, _ = call(a,'GET','/api/posts/'+pid); assert restored['title']=='An edited story'
        stop(); env['DATABASE_URL']=original_url; start()
        call(a, 'POST', '/api/posts/' + pid + '/unpublish')
        item, _ = call(public, 'GET', '/api/posts/by-slug/' + slug); assert item is None
        call(a, 'DELETE', '/api/posts/' + pid)
        mine, _ = call(a, 'GET', '/api/posts/mine'); assert mine == []
        with sqlite3.connect(Path(tmp) / 'check.db') as db:
            db.execute("UPDATE sessions SET expires_at = '2000-01-01 00:00:00' WHERE user_id = ?", (user['id'],))
        me, _ = call(a, 'GET', '/api/auth/me'); assert me is None
        call(a, 'GET', '/api/posts/mine', expected=401)
        # Limiting is based on the real peer, not spoofable forwarding headers.
        for _ in range(30):
            call(public,'POST','/api/auth/login',{'email':'missing@example.invalid','password':'invalid'},expected=401)
        _, limit_headers = call(public,'POST','/api/auth/login',{'email':'missing@example.invalid','password':'invalid'},expected=429,headers={'X-Forwarded-For':'8.8.8.8'})
        assert limit_headers['Retry-After']=='60'
        # Checksummed migrations fail closed without touching content.
        stop()
        with sqlite3.connect(Path(tmp)/'check.db') as db: db.execute("UPDATE fm_migrations SET checksum='changed' WHERE version=1")
        failed = subprocess.run([str(BINARY)],env=env,cwd=tmp,capture_output=True,timeout=10)
        assert failed.returncode != 0 and b'checksum mismatch' in failed.stderr
        with sqlite3.connect(Path(tmp)/'check.db') as db:
            assert db.execute("SELECT title FROM posts WHERE id='legacy-post'").fetchone()[0]=='Existing draft'
        print('PASS: safe Markdown, concurrency conflicts, parallel slugs, all-page search, request limits/origins, hashed legacy sessions, migration checksums, real backup restore;  skills CRUD/validation/ownership, publications CRUD/ownership, draft privacy, story assignment/moves/removal, preservation on publication delete, additive legacy migration, and persistence;  registration, invalid/valid login, cookies, logout, expiration, protected routes, ownership, story CRUD, publish/unpublish, CORS, and persistence across API restart.')
    finally:
        stop(); log.close()
