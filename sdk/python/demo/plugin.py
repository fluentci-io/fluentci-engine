from __future__ import annotations

import extism
from fluentci import pipeline


@extism.plugin_fn
def greet() -> None:
    name = extism.input_str()
    output = pipeline("python-greeting").exec("/usr/bin/printf", "%s", name).stdout()
    extism.output_str(output)
