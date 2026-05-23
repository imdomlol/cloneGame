"""Rolling system-map state for Phase 2 codegen (Layer 4 of the context filter).

Tracks ``implemented`` / ``pending`` / ``test_state`` / ``last_engine_baseline_hash``
across codegen turns so the prompt does not have to replay raw history. The
rendered YAML is hard-capped at 1,000 tokens (DEPLOYMENT_GUIDE §3.7); when
adding entries pushes past the cap, ``cap_tokens`` collapses the oldest
``implemented`` entries into a single summary line. A Haiku-driven compactor
is provided as an optional path for richer summarisation.

CLI::

    python phase2/system_map.py init
    python phase2/system_map.py implement ranger src/units/ranger.rs <sha> vault/unit/ranger.md
    python phase2/system_map.py pending soldier bow,sword
    python phase2/system_map.py tests --passing 12 --failing 3 --failing-ids ranger,bow
    python phase2/system_map.py baseline-hash <sha>
    python phase2/system_map.py show
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Any

_REPO_ROOT = Path(__file__).resolve().parent.parent
if str(_REPO_ROOT / "phase2") not in sys.path:
    sys.path.insert(0, str(_REPO_ROOT / "phase2"))

from retrieval import count_tokens  # noqa: E402

DEFAULT_PATH = _REPO_ROOT / "build" / "system_map.yaml"
TOKEN_CAP = 1000
_MIN_KEPT_IMPLEMENTED = 5

_EMPTY_STATE: dict[str, Any] = {
    "implemented": [],
    "pending": [],
    "test_state": {"passing": 0, "failing": 0, "failing_ids": []},
    "last_engine_baseline_hash": "",
}


def empty_state() -> dict[str, Any]:
    """Return a fresh deep-copy of the empty state shape."""
    import copy

    return copy.deepcopy(_EMPTY_STATE)


def load_state(path: Path) -> dict[str, Any]:
    """Load YAML state from disk; return ``empty_state()`` if the file is absent."""
    import yaml

    if not path.exists():
        return empty_state()
    raw = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    merged = empty_state()
    merged.update(raw)
    return merged


def save_state(path: Path, state: dict[str, Any]) -> None:
    """Write YAML state to ``path``, creating parent dirs as needed."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(render_yaml(state), encoding="utf-8")


def render_yaml(state: dict[str, Any]) -> str:
    """Stable YAML render; sort_keys=False so the documented section order holds."""
    import yaml

    return yaml.safe_dump(state, sort_keys=False, default_flow_style=False)


def record_implementation(
    state: dict[str, Any],
    note_id: str,
    file: str,
    sha: str,
    verified_against: str,
) -> None:
    """Append an ``implemented`` entry and drop a matching ``pending`` entry."""
    state.setdefault("implemented", []).append(
        {
            "id": note_id,
            "file": file,
            "hash": sha,
            "verified_against": verified_against,
        }
    )
    state["pending"] = [p for p in state.get("pending", []) if p.get("id") != note_id]


def record_pending(state: dict[str, Any], note_id: str, blocked_by: list[str]) -> None:
    """Add or update a ``pending`` entry. Duplicate ids overwrite (last write wins)."""
    state["pending"] = [p for p in state.get("pending", []) if p.get("id") != note_id]
    state["pending"].append({"id": note_id, "blocked_by": list(blocked_by)})


def update_test_state(
    state: dict[str, Any],
    passing: int,
    failing: int,
    failing_ids: list[str],
) -> None:
    """Overwrite the ``test_state`` block with the latest test run summary."""
    state["test_state"] = {
        "passing": passing,
        "failing": failing,
        "failing_ids": list(failing_ids),
    }


def set_baseline_hash(state: dict[str, Any], sha: str) -> None:
    """Pin the engine-baseline content hash so codegen can detect TTL invalidation."""
    state["last_engine_baseline_hash"] = sha


def cap_tokens(
    state: dict[str, Any],
    cap: int = TOKEN_CAP,
    token_counter: Any = count_tokens,
    min_kept: int = _MIN_KEPT_IMPLEMENTED,
) -> dict[str, Any]:
    """Collapse oldest ``implemented`` entries until the YAML fits ``cap`` tokens.

    Always keeps the most recent ``min_kept`` entries in full. Older entries
    are folded into one ``{summary, count}`` placeholder line so the codegen
    LLM still sees that *N earlier files exist*, just not their detail.
    """
    import copy

    work = copy.deepcopy(state)
    while token_counter(render_yaml(work)) > cap:
        impl = work.get("implemented", [])
        detailed = [e for e in impl if "summary" not in e]
        if len(detailed) <= min_kept:
            break
        first_detail_idx = next((i for i, e in enumerate(impl) if "summary" not in e), None)
        if first_detail_idx is None:
            break
        existing_summary_count = sum(int(e.get("count", 0)) for e in impl if "summary" in e)
        new_count = existing_summary_count + 1
        without_summary = [e for e in impl if "summary" not in e]
        kept = without_summary[1:]
        summary_entry = {
            "summary": "earlier implemented files collapsed",
            "count": new_count,
        }
        work["implemented"] = [summary_entry, *kept]
    return work


def summarise_with_haiku(
    state: dict[str, Any],
    client: Any,
    model: str = "claude-haiku-4-5-20251001",
    max_tokens: int = 800,
) -> dict[str, Any]:
    """Optional LLM compactor used when deterministic ``cap_tokens`` is too lossy.

    Sends the current YAML to Haiku with an instruction to rewrite it under
    the 1,000-token budget while preserving every ``id`` referenced in
    ``pending.blocked_by`` and the latest ``test_state``. Returns the
    rewritten state (parsed back from YAML).
    """
    import yaml

    prompt = (
        "Compact the following Phase 2 system_map YAML to under "
        f"{TOKEN_CAP} tokens. Preserve: every id that appears in any "
        "`pending.blocked_by` list, the full `test_state` block, and "
        "`last_engine_baseline_hash`. Collapse older `implemented` entries "
        "into a single `{summary, count}` line. Output YAML only, no prose.\n\n"
        f"{render_yaml(state)}"
    )
    response = client.messages.create(
        model=model,
        max_tokens=max_tokens,
        messages=[{"role": "user", "content": prompt}],
    )
    text = "".join(getattr(b, "text", "") for b in response.content)
    parsed = yaml.safe_load(text) or {}
    merged = empty_state()
    merged.update(parsed)
    return merged


def _parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--path", type=Path, default=DEFAULT_PATH)
    subs = parser.add_subparsers(dest="cmd", required=True)
    subs.add_parser("init")
    subs.add_parser("show")

    impl = subs.add_parser("implement")
    impl.add_argument("note_id")
    impl.add_argument("file")
    impl.add_argument("hash")
    impl.add_argument("verified_against")

    pend = subs.add_parser("pending")
    pend.add_argument("note_id")
    pend.add_argument("blocked_by", help="comma-separated ids")

    tests = subs.add_parser("tests")
    tests.add_argument("--passing", type=int, required=True)
    tests.add_argument("--failing", type=int, required=True)
    tests.add_argument("--failing-ids", default="")

    bh = subs.add_parser("baseline-hash")
    bh.add_argument("hash")

    return parser.parse_args(argv)


def _dispatch(args: argparse.Namespace) -> int:
    if args.cmd == "init":
        save_state(args.path, empty_state())
        print(f"wrote empty state to {args.path}")
        return 0
    state = load_state(args.path)
    if args.cmd == "show":
        print(render_yaml(state))
        return 0
    if args.cmd == "implement":
        record_implementation(state, args.note_id, args.file, args.hash, args.verified_against)
    elif args.cmd == "pending":
        blocked = [b.strip() for b in args.blocked_by.split(",") if b.strip()]
        record_pending(state, args.note_id, blocked)
    elif args.cmd == "tests":
        failing_ids = [i.strip() for i in args.failing_ids.split(",") if i.strip()]
        update_test_state(state, args.passing, args.failing, failing_ids)
    elif args.cmd == "baseline-hash":
        set_baseline_hash(state, args.hash)
    state = cap_tokens(state)
    save_state(args.path, state)
    tokens = count_tokens(render_yaml(state))
    print(f"system_map: {tokens}/{TOKEN_CAP} tokens, wrote {args.path}")
    return 0


def main(argv: list[str] | None = None) -> int:
    return _dispatch(_parse_args(argv))


if __name__ == "__main__":
    sys.exit(main())
