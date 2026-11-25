#!/usr/bin/env bash
# What startproject and startapp create, which files Django's runner finds and
# from which folder, the shape of a test it runs, and what a test needs first.
# usage: bash run.sh <python with django 6.1.3> <empty dir>
set -u
PY=$1
D=$2
BIN=$(dirname "$PY")
cd "$D" || exit 1
# -I leaves the folder of manage.py off sys.path, so manage() puts it there.
manage() { "$PY" -I -c "import sys, runpy; sys.path.insert(0, '$D'); sys.argv[0] = 'manage.py'; runpy.run_path('$D/manage.py', run_name='__main__')" "$@"; }
echo "== versions"
"$PY" -I -c 'import sys, django; print("python", sys.version.split()[0], "django", django.__version__)'

echo "== 1. the files of startproject and startapp"
"$BIN/django-admin" startproject mysite .
manage startapp polls
find . -type f -not -name '*.pyc' | sort
echo "-- polls/tests.py"
cat polls/tests.py

# The settings list both apps; shop has a model and a tests/ package.
manage startapp shop
sed -i "s/'django.contrib.staticfiles',/'django.contrib.staticfiles', 'polls', 'shop',/" mysite/settings.py
rm shop/tests.py
mkdir -p shop/tests polls/nopkg
touch shop/tests/__init__.py
cat >shop/models.py <<'EOF'
from django.db import models
class Book(models.Model):
    title = models.CharField(max_length=10)
EOF
cat >shop/tests/test_models.py <<'EOF'
from django.test import TestCase
from shop.models import Book
class BookTests(TestCase):
    def test_saves(self):
        Book.objects.create(title="a")
        self.assertEqual(Book.objects.count(), 1)
EOF
manage makemigrations shop >/dev/null
case_file() {
  printf 'from django.test import SimpleTestCase\nclass %s(SimpleTestCase):\n    def test_runs(self):\n        pass\n' "$1" >"$2"
}
cat >polls/tests.py <<'EOF'
from django.test import SimpleTestCase
class PollTests(SimpleTestCase):
    def test_method(self):
        pass
def test_top_level():
    raise AssertionError("a top-level function ran")
EOF
case_file ViewTests polls/test_views.py
case_file ExtraTests polls/tests_extra.py
case_file CheckTests polls/check_models.py
case_file SuffixTests polls/models_test.py
case_file NoPackageTests polls/nopkg/test_y.py
echo "== 2. the test files that the probe wrote"
find polls shop -name '*.py' -path '*test*' -o -name 'check_*.py' | sort

# The runner writes the migrations' lines and the tests' lines to one stream, so
# ran() keeps only the parts that name the database, a test or a count.
ran() { grep -oE "test_[a-z_]+ \([a-zA-Z_.]+\) \.\.\. [a-zA-Z]+|^(Found|Creating test database|Ran ).*"; }
echo "== 3. manage.py test -v 2, from the folder of manage.py"
manage test -v 2 2>&1 | ran
echo "== 4. manage.py test -v 2 -p '*_test.py'"
manage test -v 2 -p '*_test.py' 2>&1 | ran
echo "== 5. manage.py test -v 2 -p 'check_*.py'"
manage test -v 2 -p 'check_*.py' 2>&1 | ran
echo "== 6. manage.py test -v 2, from the folder polls"
(cd polls && manage test -v 2 2>&1 | ran)

# -I ignores PYTHONPATH, so each call puts the probe's folder on sys.path.
echo "== 7. unittest on polls.tests, with no settings"
"$PY" -I -c "import sys; sys.path.insert(0, '$D'); import unittest; unittest.main(module=None, argv=['x', 'polls.tests'])" 2>&1 | grep -E '^django\.|^Ran'
echo "== 8. unittest on shop.tests.test_models, with DJANGO_SETTINGS_MODULE and no django.setup()"
DJANGO_SETTINGS_MODULE=mysite.settings "$PY" -I -c "import sys; sys.path.insert(0, '$D'); import unittest; unittest.main(module=None, argv=['x', 'shop.tests.test_models'])" 2>&1 | tail -1
echo "== 9. unittest on shop.tests.test_models, after django.setup(), with no test database"
DJANGO_SETTINGS_MODULE=mysite.settings "$PY" -I -c "import sys; sys.path.insert(0, '$D'); import django; django.setup(); import unittest; unittest.main(module=None, argv=['x', 'shop.tests.test_models'])" 2>&1 | grep -E 'Error|^Ran|^OK|^FAILED' | head -3
echo "== 10. the names that test_*.py or *_test.py match, by fnmatch: a matcher, not purba"
"$PY" -I -c "
import fnmatch, os
for root, dirs, files in sorted(os.walk('.')):
    for f in sorted(files):
        if fnmatch.fnmatchcase(f, 'test_*.py') or fnmatch.fnmatchcase(f, '*_test.py'):
            print(os.path.join(root, f))
"
