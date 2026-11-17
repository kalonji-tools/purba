"""Runs every way of ways.py under every means, and writes the matrix.

    python matrix.py <python to probe> <site-packages that holds pytest> <work dir> <out>

<work dir> receives one directory for each way and a copy of site-packages
without pytest. <out> receives `<label>.json`, the raw results, and `<label>.md`,
the matrix, where <label> is the probed interpreter's version.

Static means read the text of the way and run in this process with `ast`,
which has the same node kinds as ruff's AST for each form read here. The
`modulefinder` means and each runtime means run the probed interpreter.
"""

import ast
import json
import os
import re
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
from ways import WAYS  # noqa: E402

BLOCK = ("pytest", "_pytest")
STATIC = ["stmt-scan", "known-call-scan", "name-mention-scan", "modulefinder"]
IN_PROCESS_DETECT = ["modules-diff", "audit-open", "monitoring-code"]
IN_PROCESS_BLOCK = [
    "modules-none",
    "meta-path-finder",
    "builtins-import",
    "audit-import",
    "path-without-target",
]
OTHER_PROCESS = ["importtime", "strace-open"]
DETECT = set(STATIC + IN_PROCESS_DETECT + OTHER_PROCESS)
MARK = "/nonexistent/probe-load-start"
# The way that runs pytest's code with no module object left to find.
NO_EVIDENCE = {"exec-file-text"}

# Callables that the docs say import a module named by their first argument.
KNOWN_CALLS = {
    "__import__",
    "import_module",
    "find_spec",
    "resolve_name",
    "run_module",
    "files",
    "patch",
}


def clip(text, width=80):
    return text if len(text) <= width else text[: width - 3] + "..."


def top(name):
    return re.split(r"[.:]", name)[0]


def stmt_scan(tree):
    hits = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            hits += [a.name for a in node.names if top(a.name) in BLOCK]
        elif isinstance(node, ast.ImportFrom) and node.level == 0 and top(node.module) in BLOCK:
            hits.append(node.module)
    return hits


def known_call_scan(tree):
    hits = stmt_scan(tree)
    for node in ast.walk(tree):
        if isinstance(node, ast.Call) and node.args:
            func = node.func
            name = func.id if isinstance(func, ast.Name) else getattr(func, "attr", "")
            first = node.args[0]
            if (
                name in KNOWN_CALLS
                and isinstance(first, ast.Constant)
                and isinstance(first.value, str)
                and top(first.value) in BLOCK
            ):
                hits.append(f"{name}({first.value!r})")
    return hits


def name_mention_scan(tree):
    pattern = re.compile(r"(?<![\w])_?pytest(?![\w])")
    hits = []
    for node in ast.walk(tree):
        text = None
        if isinstance(node, ast.Constant) and isinstance(node.value, (str, bytes)):
            value = node.value
            text = value.decode("latin-1") if isinstance(value, bytes) else value
        elif isinstance(node, ast.Name):
            text = node.id
        elif isinstance(node, ast.Attribute):
            text = node.attr
        elif isinstance(node, ast.alias):
            text = node.name
        elif isinstance(node, ast.ImportFrom):
            text = node.module
        if text and pattern.search(text):
            hits.append(text[:40])
    return hits


FINDER = """
import json, sys, time
from modulefinder import ModuleFinder
start = time.perf_counter()
finder = ModuleFinder(path=[sys.argv[1]] + sys.path)
finder.run_script(sys.argv[2])
found = sorted(n for n in finder.modules if n.split('.')[0] in ('pytest', '_pytest'))
bad = sorted(n for n in finder.badmodules if n.split('.')[0] in ('pytest', '_pytest'))
print(json.dumps({"hits": found, "bad": bad, "modules": len(finder.modules),
                  "seconds": round(time.perf_counter() - start, 2)}))
"""


def write_way(work, way):
    name, _, _, test, files, _ = way
    path = os.path.join(work, name)
    os.makedirs(os.path.join(path, "lib"), exist_ok=True)
    with open(os.path.join(path, "test_way.py"), "w") as f:
        f.write(test)
    for rel, text in files.items():
        with open(os.path.join(path, "lib", rel), "w") as f:
            f.write(text)
    return path


def filtered_site(site, work):
    path = os.path.join(work, "site-without-pytest")
    os.makedirs(path, exist_ok=True)
    for entry in os.listdir(site):
        if entry in ("pytest", "_pytest", "py.py") or entry.startswith("pytest-"):
            continue
        link = os.path.join(path, entry)
        if not os.path.lexists(link):
            os.symlink(os.path.join(site, entry), link)
    return path


def run_child(python, path, means, site, filtered, preload, env, prefix=()):
    args = [*prefix, python, os.path.join(HERE, "child.py"), path, means, site, filtered]
    if preload:
        args.append("--preload")
    start = time.perf_counter()
    done = subprocess.run(args, capture_output=True, text=True, env=env, timeout=120)
    seconds = round(time.perf_counter() - start, 2)
    try:
        lines = done.stdout.strip().splitlines()
        result = json.loads(lines[-2])
        exit_part = json.loads(lines[-1])
    except (IndexError, json.JSONDecodeError):
        result = {"hits": [], "evidence": [], "errors": [f"child failed: {done.stderr[-300:]}"]}
        exit_part = {"exit_hits": [], "exit_names": []}
    # What the means saw in an atexit callback after the tests. A check that
    # reads sys.modules after the tests is over by then, so it gets none.
    if means != "modules-diff":
        result["hits"] += [f"at exit: {h}" for h in exit_part["exit_hits"]]
    result["evidence"] += [f"at exit: {n}" for n in exit_part["exit_names"]]
    result["seconds"] = seconds
    return result, done.stderr


def one(python, site, filtered, work, env, way, means):
    name, _, _, test, _, preload = way
    path = os.path.join(work, name)
    if means in ("stmt-scan", "known-call-scan", "name-mention-scan"):
        tree = ast.parse(test)
        scan = {"stmt-scan": stmt_scan, "known-call-scan": known_call_scan}.get(
            means, name_mention_scan
        )
        return {"hits": scan(tree), "evidence": [], "errors": []}
    if means == "modulefinder":
        done = subprocess.run(
            [python, "-c", FINDER, os.path.join(path, "lib"), os.path.join(path, "test_way.py")],
            capture_output=True,
            text=True,
            env=env,
            timeout=300,
        )
        try:
            result = json.loads(done.stdout.strip().splitlines()[-1])
        except (IndexError, json.JSONDecodeError):
            last = done.stderr.strip().splitlines()[-1:]
            return {"hits": [], "evidence": [], "errors": [f"finder failed: {last}"]}
        result.update(evidence=[], errors=[])
        return result
    if means == "importtime":
        args = [python, "-X", "importtime", os.path.join(HERE, "child.py"), path, "none", site,
                filtered] + (["--preload"] if preload else [])
        done = subprocess.run(args, capture_output=True, text=True, env=env, timeout=120)
        after = done.stderr.split("PROBE-LOAD-START", 1)[-1]
        names = re.findall(r"^import time:\s+\d+ \|\s+\d+ \|\s+(\S+)$", after, re.M)
        return {"hits": [n for n in names if top(n) in BLOCK], "evidence": [], "errors": []}
    if means == "strace-open":
        trace = os.path.join(work, f"{name}.strace")
        args = ["strace", "-f", "-qq", "-e", "trace=openat", "-o", trace, python,
                os.path.join(HERE, "child.py"), path, "none", site, filtered]
        if preload:
            args.append("--preload")
        subprocess.run(args, capture_output=True, text=True, env=env, timeout=120)
        with open(trace) as f:
            text = f.read()
        os.remove(trace)
        after = text.split(MARK, 1)[-1]
        dirs = tuple(os.path.join(site, b) + os.sep for b in BLOCK)
        opened = re.findall(r'openat\([^"]*"([^"]+)"[^=]*= \d+', after)
        hits = sorted({os.path.relpath(p, site) for p in opened if p.startswith(dirs)})
        return {"hits": hits, "evidence": [], "errors": []}
    result, _ = run_child(python, path, means, site, filtered, preload, env)
    return result


def verdict(expect, means, result, baseline):
    """The cell of one way under one means.

    A detecting means flags a way when it recorded a hit. A blocking means
    flags it when it recorded a hit or the run raised an error that the run
    without a means did not. Evidence that pytest's code ran, or that the test
    bound pytest's module, means the block failed: "refused late" when it
    flagged the way after some of pytest's code ran, "MISSED" when it never
    flagged it. For the one way whose baseline leaves no evidence, a run with
    no hit and no new error counts as a miss.
    """
    if means == "modulefinder" and result["errors"]:
        return "crashed"
    if means in DETECT:
        flagged = bool(result["hits"])
        if expect in ("imports", "imports-at-exit", "binds"):
            return "caught" if flagged else "MISSED"
    else:
        new_errors = [e for e in result["errors"] if e not in baseline["errors"]]
        flagged = bool(result["hits"]) or bool(new_errors)
        if expect in ("imports", "imports-at-exit", "binds"):
            ran = bool(result["evidence"]) or (not baseline["evidence"] and not flagged)
            if ran and flagged:
                return "refused late"
            if ran:
                return "MISSED"
            return "refused" if flagged else "refused, silent"
    if expect == "no-import":
        return "FALSE REFUSAL" if flagged else "passed"
    if expect == "shadow":
        return "refused (not pytest)" if flagged else "passed"
    return "seen" if flagged else "not seen"


def main():
    python, site, work, out = sys.argv[1:5]
    os.makedirs(work, exist_ok=True)
    os.makedirs(out, exist_ok=True)
    filtered = filtered_site(site, work)
    version = subprocess.run(
        [python, "-c", "import sys; print('%d.%d.%d' % sys.version_info[:3])"],
        capture_output=True,
        text=True,
    ).stdout.strip()
    env = dict(os.environ)
    env.pop("PYTHONPATH", None)
    # A way such as `walk-packages` imports modules of site-packages that no
    # run compiled yet, and a probe writes nothing into the environment.
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    finds = subprocess.run(
        [python, "-c", "import importlib.util as u; print(u.find_spec('pytest') is not None)"],
        capture_output=True,
        text=True,
        env=env,
    ).stdout.strip()
    if finds != "True":
        # An interpreter with no pytest of its own reads the given site-packages.
        env["PYTHONPATH"] = site
    for way in WAYS:
        write_way(work, way)
    means_all = ["none"] + STATIC + IN_PROCESS_DETECT + IN_PROCESS_BLOCK + OTHER_PROCESS
    jobs = [(way, means) for way in WAYS for means in means_all]
    with ThreadPoolExecutor(max_workers=8) as pool:
        results = list(pool.map(lambda job: one(python, site, filtered, work, env, *job), jobs))
    raw = {}
    for (way, means), result in zip(jobs, results):
        result["hit_count"] = len(result["hits"])
        result["top_level_hits"] = sorted(
            {h for h in result["hits"] if re.sub(r"^after (load|call): ", "", h) in BLOCK}
        )
        # Each kept string fits a line of the repository's width in the JSON.
        result["hits"] = [clip(h) for h in result["hits"][:6]]
        result["evidence"] = [clip(e) for e in result["evidence"][:6]]
        result["errors"] = [clip(e) for e in result["errors"]]
        if "bad" in result:
            result["bad"] = [clip(b) for b in result["bad"]]
        raw.setdefault(way[0], {})[means] = result
    with open(os.path.join(out, f"{version}.json"), "w") as f:
        json.dump({"python": version, "results": raw}, f, indent=1, sort_keys=True)
        f.write("\n")
    columns = STATIC + IN_PROCESS_DETECT + IN_PROCESS_BLOCK + OTHER_PROCESS
    lines = [f"# Matrix, CPython {version}", "", "| way | does | " + " | ".join(columns) + " |",
             "|---|---|" + "---|" * len(columns)]
    for way in WAYS:
        name, group, expect = way[0], way[1], way[2]
        baseline = raw[name]["none"]
        if expect == "imports" and not baseline["evidence"] and name not in NO_EVIDENCE:
            # The way imports nothing on this version: annotations are lazy
            # from 3.14 on.
            expect = "no-import"
        cells = [verdict(expect, m, raw[name][m], baseline) for m in columns]
        lines.append(f"| {group} `{name}` | {expect} | " + " | ".join(cells) + " |")
    lines += ["", "## Baseline evidence (means `none`)", ""]
    for way in WAYS:
        b = raw[way[0]]["none"]
        lines.append(f"- `{way[0]}`: evidence {b['evidence'][:4]}, errors {b['errors']}")
    with open(os.path.join(out, f"{version}.md"), "w") as f:
        f.write("\n".join(lines) + "\n")
    print("\n".join(lines))


if __name__ == "__main__":
    main()
