import json
import sys

print(json.dumps({"message": "hello from uv", "args": sys.argv[1:]}, ensure_ascii=False))
