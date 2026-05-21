"""YAML frontmatter parsing for compiled vault notes.

Hand-rolled to handle the constrained subset of YAML the compile prompt
produces (flat scalars, nested maps, flow sequences for depends_on).
Switch to a full YAML library only if the compile contract drifts.
"""

from __future__ import annotations

import json
import re
from typing import Any


def repair_frontmatter_delimiter(markdown: str) -> str:
    """Insert a missing closing `---` before the first heading, if needed.

    The compile LLM occasionally drops the closing delimiter; this restores
    it so downstream parsing succeeds.
    """
    if re.match(r"\A---\s*\n(.*?)\n---\s*(?:\n|\Z)", markdown, re.S):
        return markdown
    if not markdown.startswith("---"):
        return markdown

    heading = re.search(r"\n(#{1,6}\s+)", markdown)
    if heading is None:
        return markdown
    insert_at = heading.start() + 1
    return markdown[:insert_at] + "---\n" + markdown[insert_at:]


def replace_frontmatter_type(markdown: str, kind: str) -> str:
    """Set the `type:` field in the YAML block to the given kind."""
    markdown = repair_frontmatter_delimiter(markdown)
    match = re.match(r"\A---\s*\n(.*?)\n---\s*(?:\n|\Z)", markdown, re.S)
    if not match:
        return markdown

    body = match.group(1)
    lines = body.splitlines()
    replaced = False
    for idx, line in enumerate(lines):
        if re.match(r"^type\s*:", line):
            lines[idx] = f"type: {json.dumps(kind)}"
            replaced = True
            break
    if not replaced:
        lines.append(f"type: {json.dumps(kind)}")

    return "---\n" + "\n".join(lines) + "\n---\n" + markdown[match.end() :]


def frontmatter(markdown: str) -> tuple[dict[str, Any], list[str]]:
    """Return parsed frontmatter dict and any structural errors."""
    markdown = repair_frontmatter_delimiter(markdown)
    match = re.match(r"\A---\s*\n(.*?)\n---\s*(?:\n|\Z)", markdown, re.S)
    if not match:
        return {}, ["missing YAML frontmatter"]
    return parse_yaml_map(match.group(1)), []


def parse_scalar(raw: str) -> Any:
    raw = raw.strip()
    if raw in {"", "null", "~"}:
        return None
    if raw in {"true", "false"}:
        return raw == "true"
    if raw.startswith(('"', "'")) and raw.endswith(raw[0]):
        return raw[1:-1]
    if raw.startswith("[") or raw.startswith("{"):
        try:
            return json.loads(raw.replace("'", '"'))
        except Exception:
            if raw.startswith("[") and raw.endswith("]"):
                return parse_flow_sequence(raw)
            return raw
    try:
        return int(raw) if re.fullmatch(r"-?\d+", raw) else float(raw)
    except ValueError:
        return raw


def parse_flow_sequence(raw: str) -> list[Any]:
    body = raw[1:-1].strip()
    if not body:
        return []

    items: list[str] = []
    current: list[str] = []
    quote = ""
    escaped = False
    for char in body:
        if escaped:
            current.append(char)
            escaped = False
            continue
        if char == "\\" and quote == '"':
            current.append(char)
            escaped = True
            continue
        if quote:
            current.append(char)
            if char == quote:
                quote = ""
            continue
        if char in {"'", '"'}:
            quote = char
            current.append(char)
            continue
        if char == ",":
            items.append("".join(current).strip())
            current = []
            continue
        current.append(char)
    items.append("".join(current).strip())
    return [parse_scalar(item) for item in items if item]


def parse_yaml_map(text: str) -> dict[str, Any]:
    lines = [line.rstrip() for line in text.splitlines() if line.strip()]
    data, _ = parse_block(lines, 0, 0)
    return data if isinstance(data, dict) else {}


def line_indent(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def parse_block(lines: list[str], idx: int, indent: int) -> tuple[Any, int]:
    if idx >= len(lines):
        return {}, idx
    if lines[idx].strip().startswith("- "):
        return parse_list(lines, idx, indent)
    return parse_map(lines, idx, indent)


def parse_map(lines: list[str], idx: int, indent: int) -> tuple[dict[str, Any], int]:
    out: dict[str, Any] = {}
    while idx < len(lines) and line_indent(lines[idx]) >= indent:
        if line_indent(lines[idx]) != indent or lines[idx].strip().startswith("- "):
            break
        key, _, val = lines[idx].strip().partition(":")
        idx += 1
        if val.strip():
            out[key] = parse_scalar(val)
        elif idx < len(lines) and line_indent(lines[idx]) > indent:
            out[key], idx = parse_block(lines, idx, line_indent(lines[idx]))
        else:
            out[key] = {}
    return out, idx


def parse_list(lines: list[str], idx: int, indent: int) -> tuple[list[Any], int]:
    out: list[Any] = []
    while idx < len(lines) and line_indent(lines[idx]) == indent:
        item = lines[idx].strip()[2:].strip()
        idx += 1
        if ":" in item and not item.startswith(('"', "'")):
            key, _, val = item.partition(":")
            obj = {key: parse_scalar(val)} if val.strip() else {key: {}}
            if idx < len(lines) and line_indent(lines[idx]) > indent:
                extra, idx = parse_map(lines, idx, line_indent(lines[idx]))
                obj.update(extra)
            out.append(obj)
        elif item:
            out.append(parse_scalar(item))
    return out, idx
