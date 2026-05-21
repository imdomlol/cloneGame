"""MediaWiki API helpers: category enumeration and page wikitext retrieval.

Retries 429 + 5xx with exponential backoff. All callers must supply a
user-agent string per Fandom's API etiquette.
"""

from __future__ import annotations

import json
import time
import urllib.error
import urllib.parse
import urllib.request
from typing import Any


def build_url(api_endpoint: str, params: dict[str, object]) -> str:
    return f"{api_endpoint}?{urllib.parse.urlencode(params, doseq=True)}"


def http_get_json(url: str, user_agent: str, timeout_s: float = 30.0) -> dict[str, Any]:
    req = urllib.request.Request(url, headers={"User-Agent": user_agent})
    with urllib.request.urlopen(req, timeout=timeout_s) as resp:
        return json.loads(resp.read().decode("utf-8"))


def request_json(url: str, user_agent: str, retry_count: int) -> dict[str, Any]:
    for attempt in range(retry_count + 1):
        try:
            return http_get_json(url, user_agent)
        except urllib.error.HTTPError as exc:
            if exc.code != 429 and exc.code < 500:
                raise
            if attempt >= retry_count:
                raise
        except (urllib.error.URLError, TimeoutError):
            if attempt >= retry_count:
                raise
        time.sleep(min(2**attempt, 30))
    raise RuntimeError("unreachable retry loop")


def category_members(
    api: str, category: str, user_agent: str, retries: int
) -> list[dict[str, Any]]:
    params: dict[str, object] = {
        "action": "query",
        "list": "categorymembers",
        "cmtitle": f"Category:{category}",
        "cmlimit": "max",
        "cmtype": "page",
        "format": "json",
    }
    out: list[dict[str, Any]] = []
    while True:
        data = request_json(build_url(api, params), user_agent, retries)
        members = data.get("query", {}).get("categorymembers", [])
        out.extend([m for m in members if isinstance(m, dict)])
        cont = data.get("continue")
        if not isinstance(cont, dict) or not cont:
            return out
        params.update(cont)


def page_wikitext(api: str, title: str, user_agent: str, retries: int) -> tuple[str, str]:
    params = {
        "action": "query",
        "prop": "revisions",
        "titles": title,
        "rvprop": "ids|content",
        "rvslots": "main",
        "formatversion": "2",
        "format": "json",
    }
    data = request_json(build_url(api, params), user_agent, retries)
    pages = data.get("query", {}).get("pages", [])
    if not pages or "missing" in pages[0]:
        raise ValueError(f"missing wiki page: {title}")
    rev = pages[0].get("revisions", [{}])[0]
    main_slot = rev.get("slots", {}).get("main", {})
    text = main_slot.get("content", rev.get("*", ""))
    return str(text), str(rev.get("revid", "unknown"))
