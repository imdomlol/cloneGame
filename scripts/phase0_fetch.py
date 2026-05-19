"""
Phase 0 - Taxonomy discovery

Stdlib-only MediaWiki API crawler for deriving a gameplay taxonomy seed from a
Fandom wiki's category graph.
"""

from __future__ import annotations

import argparse
import json
import sys
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

_FILTER_SUBSTRINGS = [
    "stub",
    "cleanup",
    "delete",
    "orphan",
    "maintenance",
    "template",
    "file",
    "image",
    "video",
    "redirect",
    "navbox",
    "infobox",
    "disambig",
    "tracking",
    "hidden",
    "talk",
    "user",
    "mediawiki",
    "help",
    "portal",
    "wikia",
    "fandom",
    "policy",
    "candidate",
    "nomination",
    "request",
    "archive",
]


def _load_user_agent_from_phase1_config(repo_root: Path) -> str | None:
    path = repo_root / "phase1.config.toml"
    if not path.exists():
        return None

    try:
        import tomllib  # py>=3.11
    except Exception:
        return None

    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None

    ingest = data.get("ingest")
    if not isinstance(ingest, dict):
        return None

    ua = ingest.get("user_agent")
    if isinstance(ua, str) and ua.strip():
        return ua.strip()
    return None


def _api_url(wiki_base_url: str) -> str:
    # wiki_base_url typically includes a /wiki/ path segment (e.g. .../wiki/).
    # The MediaWiki API lives at the domain root (/api.php), not under /wiki/.
    base = wiki_base_url.rstrip("/")
    if base.endswith("/wiki"):
        base = base[: -len("/wiki")]
    return base + "/api.php"


def _http_get_json(
    url: str,
    *,
    user_agent: str | None,
    timeout_s: float = 20.0,
) -> dict:
    headers = {}
    if user_agent:
        headers["User-Agent"] = user_agent

    req = urllib.request.Request(url, headers=headers)
    with urllib.request.urlopen(req, timeout=timeout_s) as resp:
        raw = resp.read()
    return json.loads(raw.decode("utf-8"))


def _build_url(api_endpoint: str, params: dict[str, object]) -> str:
    query = urllib.parse.urlencode(params, doseq=True)
    return f"{api_endpoint}?{query}"


def _is_maintenance_category(name: str) -> bool:
    lowered = name.casefold()
    return any(substr in lowered for substr in _FILTER_SUBSTRINGS)


def _query_all_categories(
    api_endpoint: str,
    *,
    min_members: int,
    user_agent: str | None,
) -> list[dict]:
    params: dict[str, object] = {
        "action": "query",
        "list": "allcategories",
        "aclimit": "max",
        "acminsize": str(int(min_members)),
        "acprop": "size",
        "format": "json",
    }

    out: list[dict] = []
    while True:
        url = _build_url(api_endpoint, params)
        data = _http_get_json(url, user_agent=user_agent)

        query = data.get("query")
        if isinstance(query, dict):
            allcats = query.get("allcategories")
            if isinstance(allcats, list):
                out.extend([c for c in allcats if isinstance(c, dict)])

        cont = data.get("continue")
        if isinstance(cont, dict) and cont:
            for k, v in cont.items():
                params[k] = v
            continue

        legacy_cont = data.get("query-continue")
        if isinstance(legacy_cont, dict) and legacy_cont:
            # Old MediaWiki format; carry forward the continuation keys.
            for sub in legacy_cont.values():
                if isinstance(sub, dict):
                    for k, v in sub.items():
                        params[k] = v
            continue

        break

    return out


def _query_category_members(
    api_endpoint: str,
    category_name: str,
    *,
    user_agent: str | None,
    limit: int = 10,
) -> list[str]:
    params: dict[str, object] = {
        "action": "query",
        "list": "categorymembers",
        "cmtitle": f"Category:{category_name}",
        "cmlimit": str(int(limit)),
        "cmtype": "page",
        "format": "json",
    }
    url = _build_url(api_endpoint, params)
    data = _http_get_json(url, user_agent=user_agent)

    query = data.get("query")
    if not isinstance(query, dict):
        return []

    cms = query.get("categorymembers")
    if not isinstance(cms, list):
        return []

    titles: list[str] = []
    for item in cms:
        if not isinstance(item, dict):
            continue
        title = item.get("title")
        if isinstance(title, str) and title:
            titles.append(title)
    return titles


def fetch_taxonomy(wiki_base_url: str, min_members: int = 3) -> list[dict]:
    """
    Query the MediaWiki API for primary gameplay categories, filter out
    maintenance categories, and attach up to 2 seed page titles per category.

    Returns:
        [{"name": str, "member_count": int, "members": list[str]}]
    """
    repo_root = Path(__file__).resolve().parents[1]
    user_agent = _load_user_agent_from_phase1_config(repo_root)

    api_endpoint = _api_url(wiki_base_url)
    allcats = _query_all_categories(
        api_endpoint,
        min_members=min_members,
        user_agent=user_agent,
    )

    filtered: list[dict] = []
    for cat in allcats:
        name = cat.get("*")
        if not isinstance(name, str) or not name:
            continue
        if _is_maintenance_category(name):
            continue

        size = cat.get("size")
        try:
            member_count = int(size)
        except Exception:
            member_count = 0

        filtered.append(
            {
                "name": name,
                "member_count": member_count,
                "members": [],
            }
        )

    for item in filtered:
        item["members"] = _query_category_members(
            api_endpoint,
            item["name"],
            user_agent=user_agent,
            limit=10,
        )

    filtered.sort(key=lambda d: int(d.get("member_count", 0)), reverse=True)
    return filtered


def _load_default_wiki_base_url(repo_root: Path) -> str:
    path = repo_root / "game-config.json"
    data = json.loads(path.read_text(encoding="utf-8"))
    game = data.get("game")
    if not isinstance(game, dict):
        raise ValueError("game-config.json missing required object: game")
    url = game.get("wiki_base_url")
    if not isinstance(url, str) or not url.strip():
        raise ValueError("game-config.json missing required string: game.wiki_base_url")
    return url.strip()


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Fetch a filtered wiki category taxonomy via MediaWiki API."
    )
    parser.add_argument(
        "wiki_base_url",
        nargs="?",
        help="Wiki base URL (e.g. https://.../wiki/). Defaults to game-config.json.",
    )
    args = parser.parse_args(argv)

    repo_root = Path(__file__).resolve().parents[1]
    wiki_base_url = args.wiki_base_url or _load_default_wiki_base_url(repo_root)

    try:
        result = fetch_taxonomy(wiki_base_url)
    except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError) as exc:
        print(f"MediaWiki API unreachable for {wiki_base_url!r}: {exc}", file=sys.stderr)
        return 1
    except Exception as exc:
        print(str(exc), file=sys.stderr)
        return 1

    json.dump(result, sys.stdout, ensure_ascii=False, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
