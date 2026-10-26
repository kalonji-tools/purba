import os
import subprocess
import sys

subprocess.run(["lsof", "-a", "-p", str(os.getpid()), "-d", "0-9"], stdout=2, check=False)
print("env:", {k: v for k, v in os.environ.items() if "PYTHON" in k}, file=sys.stderr)
