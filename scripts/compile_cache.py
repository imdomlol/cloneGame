"""Compile prompt rendering, headless LLM dispatch, and SHA-256 cache.

`cached_or_compile` keys cache files on the FULL rendered prompt plus
model id so any change to wikitext, the system prompt, or a kind's
frontmatter_schema invalidates the cache cleanly.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import subprocess
from pathlib import Path
from typing import Any


def compile_prompt(template: str, kind: str, source: str, schema: dict[str, Any]) -> str:
    """Render the compile prompt with the per-kind frontmatter schema inlined."""
    schema_json = json.dumps(schema, indent=2, ensure_ascii=False) if schema else "{}"
    prompt = template.replace("{{type_hint}}", kind)
    prompt = prompt.replace("{{kind_schema}}", schema_json)
    return prompt.replace("{{stripped_html}}", source)


def cache_key(rendered_prompt: str, model: str) -> str:
    """Hash the final rendered prompt + model id (see module docstring)."""
    h = hashlib.sha256()
    h.update(rendered_prompt.encode("utf-8"))
    h.update(b"\0")
    h.update(model.encode("utf-8"))
    return h.hexdigest()


def run_llm(prompt: str, mode: str, model: str) -> str:
    if mode == "claude":
        # --tools "" disables all built-in tools. Without it, `claude -p` behaves
        # like an interactive agent: it tries to Write/Edit the output, gets
        # blocked by the sandbox, and inconsistently falls back to either inline
        # code or a plan-only summary ("grant permissions and I'll write..."). A
        # pure text transform must not have tools, so the model just emits text.
        cmd = ["claude", "-p", "--model", model, "--tools", ""]
    elif mode == "codex":
        codex_cmd = "codex.cmd" if os.name == "nt" else "codex"
        cmd = [codex_cmd, "exec", "--model", model, "-"]
    else:
        raise ValueError(f"unsupported [compile] llm_mode: {mode}")
    proc = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
    )
    try:
        stdout, stderr = proc.communicate(prompt)
    except KeyboardInterrupt:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
            proc.wait()
        raise
    if proc.returncode != 0:
        raise RuntimeError(stderr.strip() or f"{cmd[0]} exited {proc.returncode}")
    return stdout


def cached_or_compile(
    cache_dir: Path, key: str, prompt: str, mode: str, model: str
) -> tuple[str, str]:
    path = cache_dir / f"{key}.md"
    if path.exists():
        return path.read_text(encoding="utf-8"), "cached"
    output = run_llm(prompt, mode, model)
    cache_dir.mkdir(parents=True, exist_ok=True)
    path.write_text(output, encoding="utf-8")
    return output, "compiled"


def strip_llm_chatter(markdown: str) -> str:
    """Drop preamble before the first '---' and a trailing ``` fence.

    Agentic CLIs sometimes wrap output in "I'll convert this..." or
    ```markdown ... ``` fences despite a strict-output prompt.
    """
    lines = markdown.splitlines(keepends=True)
    for i, line in enumerate(lines):
        if line.strip() == "---":
            stripped = "".join(lines[i:])
            return re.sub(r"\n```\s*\Z", "\n", stripped)
    return markdown
