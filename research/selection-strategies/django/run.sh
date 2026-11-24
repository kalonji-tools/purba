#!/usr/bin/env bash
# Django finds its units through a manifest (INSTALLED_APPS), then a folder
# (management/commands/) and a file name (admin.py), then a call that
# registers each unit (admin.site.register). Flask registers each route by a call.
# usage: bash run.sh <python with django 6.1.3 and flask 3.1.3> <empty dir>
set -u
PY=$1
D=$2
cd "$D" || exit 1
echo "== versions"
"$PY" -I -c 'import django, importlib.metadata as m; print("django", django.__version__, "flask", m.version("flask"))'

mkdir -p proj shop/management/commands lost/management/commands
touch shop/__init__.py shop/management/__init__.py shop/management/commands/__init__.py
touch lost/__init__.py lost/management/__init__.py lost/management/commands/__init__.py
cat >proj/settings.py <<'EOF'
SECRET_KEY = "x"
INSTALLED_APPS = ["django.contrib.admin", "django.contrib.auth", "django.contrib.contenttypes",
                  "django.contrib.sessions", "django.contrib.messages", "shop"]
DATABASES = {}
EOF
touch proj/__init__.py
cat >shop/management/commands/lend.py <<'EOF'
from django.core.management.base import BaseCommand
class Command(BaseCommand):
    def handle(self, *a, **k): self.stdout.write("lent")
EOF
printf 'def util(): return 1\n' >shop/management/commands/_util.py
printf 'def util(): return 1\n' >shop/management/commands/helper.py
cp shop/management/commands/lend.py lost/management/commands/find.py
cat >shop/models.py <<'EOF'
from django.db import models
class Book(models.Model):
    title = models.CharField(max_length=10)
class Shelf(models.Model):
    name = models.CharField(max_length=10)
EOF
cat >shop/admin.py <<'EOF'
from django.contrib import admin
from .models import Book
admin.site.register(Book)
EOF

# -I ignores PYTHONPATH, so each call puts the probe's folder on sys.path.
run() { DJANGO_SETTINGS_MODULE=proj.settings "$PY" -I -c "import sys; sys.path.insert(0, '$D'); $1" 2>&1; }

echo "== 1. the commands of the app shop: each module of management/commands/ but _util"
run 'import django; django.setup(); from django.core.management import get_commands; print(sorted(k for k, v in get_commands().items() if v in ("shop", "lost")))'
echo "== 2. run the helper module as a command"
run 'import django; django.setup(); from django.core.management import call_command; call_command("helper")' | tail -1
echo "== 3. a command of the app lost, which INSTALLED_APPS does not list"
run 'import django; django.setup(); from django.core.management import call_command; call_command("find")' | tail -1
echo "== 4. the admin's registry after autodiscover: Book is in, Shelf is not"
run 'import django; django.setup(); from django.contrib import admin; print(sorted(m.__name__ for m in admin.site._registry if m._meta.app_label == "shop"))'
echo "== 5. a second admin.site.register(Book)"
printf 'admin.site.register(Book)\n' >>shop/admin.py
run 'import django; django.setup()' | tail -1

echo "== 6. Flask: two routes registered under one endpoint name"
cat >app.py <<'EOF'
from flask import Flask
app = Flask(__name__)

def page():
    return "a"

app.add_url_rule("/a", "page", page)
try:
    app.add_url_rule("/b", "page", lambda: "b")
except AssertionError as e:
    print("AssertionError:", e)
print(sorted(r.rule for r in app.url_map.iter_rules()))
EOF
"$PY" -I app.py 2>&1
