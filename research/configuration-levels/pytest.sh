#!/usr/bin/env bash
# Probe: the order of pytest's levels, and of pytest-timeout's and of filterwarnings.
# Usage: pytest.sh <venv-bin-dir> <empty-work-dir>
set -u
BIN=$1
W=$2
mkdir -p "$W"
cd "$W" || exit 1
export HOME="$W/home" XDG_CONFIG_HOME="$W/home/.config"
mkdir -p "$HOME"
unset PYTEST_ADDOPTS PYTEST_TIMEOUT PYTEST_PLUGINS PYTEST_DISABLE_PLUGIN_AUTOLOAD
run() {
  echo "\$ $*"
  "$@" 2>&1 | grep -E "^(configfile|rootdir)|passed|failed|Timeout|error" | head -4
  echo
}

echo "## 1. which file is found: walk up from the argument, first match wins"
mkdir -p walk/a/b
printf '[project]\nname="x"\n' >walk/pyproject.toml # a pyproject.toml with no pytest table
printf '[pytest]\nmarkers =\n  fromini: in walk/pytest.ini\n' >walk/pytest.ini
printf '[tool.pytest.ini_options]\nmarkers = ["frompyproject: in walk/a"]\n' >walk/a/pyproject.toml
printf '[tool:pytest]\nmarkers =\n  fromsetup: in walk/a/b\n' >walk/a/b/setup.cfg
printf 'def test_x(): pass\n' >walk/a/b/test_w.py
run "$BIN/pytest" -p no:cacheprovider walk/a/b/test_w.py
rm walk/a/b/setup.cfg
printf '[metadata]\nname = x\n' >walk/a/b/setup.cfg # a setup.cfg with no pytest section
run "$BIN/pytest" -p no:cacheprovider walk/a/b/test_w.py
rm walk/a/pyproject.toml
printf '[project]\nname="y"\n' >walk/a/pyproject.toml # no table: does it stop the walk?
run "$BIN/pytest" -p no:cacheprovider walk/a/b/test_w.py
echo "same, but from a cwd with no file (cwd is not the start of the walk):"
(cd / && run "$BIN/pytest" -p no:cacheprovider "$W/walk/a/b/test_w.py")

echo "## 2. a scalar option: ini addopts, then PYTEST_ADDOPTS, then the command line; the last one wins"
mkdir -p order && cd order
printf '[pytest]\naddopts = --maxfail=1\n' >pytest.ini
cat >test_o.py <<'EOF'
def test_a(): assert 0, "a fails"
def test_b(): assert 0, "b fails"
def test_c(): assert 0, "c fails"
EOF
run "$BIN/pytest" -p no:cacheprovider -q
PYTEST_ADDOPTS="--maxfail=2" run "$BIN/pytest" -p no:cacheprovider -q
PYTEST_ADDOPTS="--maxfail=2" run "$BIN/pytest" -p no:cacheprovider -q --maxfail=3
echo "-o addopts= on the command line (does it clear the ini addopts?)"
run "$BIN/pytest" -p no:cacheprovider -q -o addopts=
echo "-o overrides an ini value: -o markers?  use -o xfail_strict"
cd ..

echo "## 3. a list option: -W from ini addopts, PYTEST_ADDOPTS and the command line all apply"
mkdir -p lists && cd lists
printf '[pytest]\naddopts = -W error::DeprecationWarning\n' >pytest.ini
cat >test_l.py <<'EOF'
import warnings
def test_dep(): warnings.warn("d", DeprecationWarning)
def test_user(): warnings.warn("u", UserWarning)
def test_rt(): warnings.warn("r", RuntimeWarning)
EOF
PYTEST_ADDOPTS="-W error::UserWarning" run "$BIN/pytest" -p no:cacheprovider -q -W error::RuntimeWarning
cd ..

echo "## 4. pytest-timeout: ini 3 s, PYTEST_TIMEOUT, --timeout 1, marker 6 s; the test sleeps 2 s"
mkdir -p timeout && cd timeout
printf '[pytest]\ntimeout = 3\n' >pytest.ini
cat >test_t.py <<'EOF'
import time, pytest
def test_plain(): time.sleep(2)
@pytest.mark.timeout(6)
def test_marked(): time.sleep(2)
EOF
echo "ini 3 alone: both pass"
run "$BIN/pytest" -p no:cacheprovider -q
echo "ini 3 + PYTEST_TIMEOUT=1: env beats ini?"
PYTEST_TIMEOUT=1 run "$BIN/pytest" -p no:cacheprovider -q
echo "PYTEST_TIMEOUT=1 + --timeout 5: CLI beats env?"
PYTEST_TIMEOUT=1 run "$BIN/pytest" -p no:cacheprovider -q --timeout 5
echo "ini 3 + --timeout 1 + marker 6: the scenario of #1212 (project 10, code 60, CLI 5), scaled"
run "$BIN/pytest" -p no:cacheprovider -q --timeout 1
echo "module pytestmark 1 s + function marker 6 s: closest marker wins?"
cat >test_t2.py <<'EOF'
import time, pytest
pytestmark = pytest.mark.timeout(1)
@pytest.mark.timeout(6)
def test_fn_marker(): time.sleep(2)
def test_module_marker(): time.sleep(2)
EOF
run "$BIN/pytest" -p no:cacheprovider -q test_t2.py
rm test_t2.py
cd ..

echo "## 5. filterwarnings: ini, -W, and the mark"
mkdir -p fw && cd fw
printf '[pytest]\nfilterwarnings =\n    error::UserWarning\n' >pytest.ini
cat >test_f.py <<'EOF'
import warnings, pytest
def test_plain(): warnings.warn("u", UserWarning)
@pytest.mark.filterwarnings("ignore::UserWarning")
def test_marked_ignore(): warnings.warn("u", UserWarning)
EOF
echo "ini error: plain fails, marked passes (mark beats ini)"
run "$BIN/pytest" -p no:cacheprovider -q
echo "ini error + -W ignore::UserWarning: does -W beat ini?"
run "$BIN/pytest" -p no:cacheprovider -q -W ignore::UserWarning
printf '[pytest]\nfilterwarnings =\n    ignore::UserWarning\n' >pytest.ini
cat >test_f.py <<'EOF'
import warnings, pytest
def test_plain(): warnings.warn("u", UserWarning)
@pytest.mark.filterwarnings("ignore::UserWarning")
def test_marked_ignore(): warnings.warn("u", UserWarning)
EOF
echo "ini ignore + -W error::UserWarning: plain fails if -W beats ini; marked passes if mark beats -W"
run "$BIN/pytest" -p no:cacheprovider -q -W error::UserWarning
cd ..

echo "## 6. -c names the file, --rootdir and -o override ini"
mkdir -p cflag && cd cflag
printf '[pytest]\nxfail_strict = true\n' >pytest.ini
printf '[pytest]\nxfail_strict = false\n' >other.ini
cat >test_x.py <<'EOF'
import pytest
@pytest.mark.xfail
def test_xp(): pass
EOF
run "$BIN/pytest" -p no:cacheprovider -q
run "$BIN/pytest" -p no:cacheprovider -q -c other.ini
run "$BIN/pytest" -p no:cacheprovider -q -o xfail_strict=false
echo "marker strict=False beats ini xfail_strict=true?"
cat >test_x.py <<'EOF'
import pytest
@pytest.mark.xfail(strict=False)
def test_xp(): pass
EOF
run "$BIN/pytest" -p no:cacheprovider -q
cd ..

echo "## 7. a user-level file: pytest reads none; a file in HOME is not found from a dir under HOME"
mkdir -p "$HOME/proj"
printf '[pytest]\naddopts = --maxfail=1\nmarkers =\n  fromhome: x\n' >"$HOME/pytest.ini"
printf 'def test_x(): pass\n' >"$HOME/proj/test_h.py"
(cd "$HOME/proj" && run "$BIN/pytest" -p no:cacheprovider test_h.py)
echo "but a file in a parent dir IS found, so HOME is just a parent dir; outside HOME it is not:"
mkdir -p "$W/outside"
printf 'def test_x(): pass\n' >"$W/outside/test_h.py"
(cd "$W/outside" && run "$BIN/pytest" -p no:cacheprovider test_h.py)
