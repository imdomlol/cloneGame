"""Wikitext trimming helpers for the Phase 1 compile prompt.

Removes presentation cruft (HTML comments, category/file/image links,
nav/footer templates, image-only galleries, repeated blank lines) while
keeping infoboxes, stat tables, formulas, and inline wikilinks intact so
the compile LLM sees a token-lean but content-complete page.
"""

from __future__ import annotations

import re


def _remove_wikilinks_by_prefix(text: str, prefixes: tuple[str, ...]) -> str:
    lower = text.casefold()
    out: list[str] = []
    i = 0
    length = len(text)
    while i < length:
        if i + 2 <= length and text[i : i + 2] == "[[":
            prefix_match = False
            for prefix in prefixes:
                start = i + 2
                end = start + len(prefix)
                if end <= length and lower[start:end] == prefix:
                    prefix_match = True
                    break
            if prefix_match:
                depth = 1
                j = i + 2
                while j < length and depth > 0:
                    if j + 2 <= length and text[j : j + 2] == "[[":
                        depth += 1
                        j += 2
                        continue
                    if j + 2 <= length and text[j : j + 2] == "]]":
                        depth -= 1
                        j += 2
                        continue
                    j += 1
                i = j
                continue
        out.append(text[i])
        i += 1
    return "".join(out)


def _is_image_listing_line(line: str) -> bool:
    stripped = line.strip()
    if not stripped:
        return True
    lowered = stripped.casefold()
    return (
        lowered.startswith("[[file:")
        or lowered.startswith("[[image:")
        or lowered.startswith("file:")
        or lowered.startswith("image:")
    )


def trim_wikitext(text: str) -> str:
    """Strip presentation-only wikitext; keep semantic content for the LLM."""
    trimmed = re.sub(r"<!--.*?-->", "", text, flags=re.S)
    trimmed = _remove_wikilinks_by_prefix(trimmed, ("category:", "file:", "image:"))

    template_keywords = (
        "nav",
        "navbar",
        "navigation",
        "footer",
        "stub",
        "cleanup",
        "spoiler",
        "toc",
        "clear",
        "wikia",
        "fandom",
    )
    lines: list[str] = []
    for line in trimmed.splitlines():
        stripped = line.strip()
        lowered = stripped.casefold()
        is_template_line = lowered.startswith("{{") and lowered.endswith("}}")
        if is_template_line and any(keyword in lowered for keyword in template_keywords):
            continue
        lines.append(line)
    trimmed = "\n".join(lines)

    gallery_pattern = re.compile(r"<gallery\b[^>]*>(.*?)</gallery>", re.I | re.S)

    def _replace_gallery(match: re.Match[str]) -> str:
        body = match.group(1)
        if all(_is_image_listing_line(line) for line in body.splitlines()):
            return ""
        return match.group(0)

    trimmed = gallery_pattern.sub(_replace_gallery, trimmed)
    trimmed = re.sub(r"\n[ \t]*\n(?:[ \t]*\n)+", "\n\n", trimmed)
    return trimmed.strip()
