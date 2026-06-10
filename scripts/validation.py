"""Frontmatter validation against the universal + per-kind schemas.

The universal schema (`schemas/_universal.schema.json`) is the only
presence gate; per-kind schemas only type-check fields that happen to
be present. Falls back to required-field checks if `jsonschema` is
missing so dry runs and cache-safe ingest still work.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

try:
    import jsonschema
except Exception:  # pragma: no cover - dependency is optional at import time
    jsonschema = None

try:
    from referencing import Registry, Resource
    from referencing.jsonschema import DRAFT202012
except Exception:  # pragma: no cover - dependency follows modern jsonschema
    Registry = None
    Resource = None
    DRAFT202012 = None


def _read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def _plural_variants_token(token: str) -> list[str]:
    """English-ish plural/singular variants of a single word."""
    variants: list[str] = []
    if token.endswith("ies") and len(token) > 3:
        variants.append(token[:-3] + "y")
    if token.endswith("es") and len(token) > 2:
        variants.append(token[:-2])
    if token.endswith("s") and len(token) > 1:
        variants.append(token[:-1])
    if token.endswith("y") and len(token) > 1:
        variants.append(token[:-1] + "ies")
    if token.endswith(("s", "x", "z")) or token.endswith(("ch", "sh")):
        variants.append(token + "es")
    variants.append(token + "s")
    return variants


def _kind_variants(value: str) -> list[str]:
    """Plural/singular variants of a snake_case kind, varying only the last segment."""
    prefix, sep, last = value.rpartition("_")
    return [
        (prefix + sep + variant) if sep else variant for variant in _plural_variants_token(last)
    ]


def canonical_kind(value: str, kinds: set[str]) -> str | None:
    """Map a (possibly plural-mismatched) type value to a configured kind name.

    Tries exact match first, then simple English plural/singular swaps on the
    last snake_case segment (e.g. `game_mechanic` -> `game_mechanics`,
    `mayors` -> `mayor`). Returns None if no variant matches — caller should
    surface that as a normal validation error so the user can fix the config.
    """
    if not value:
        return None
    if value in kinds:
        return value
    for variant in _kind_variants(value):
        if variant in kinds:
            return variant
    return None


def raw_kind_schema(game_config: dict[str, Any], kind: str) -> dict[str, Any]:
    """Return the per-kind frontmatter schema as authored in game-config.json.

    Used both for prompt injection (so the LLM sees canonical field names)
    and, via `kind_frontmatter_schema`, for validation.
    """
    kinds = game_config.get("kinds", {})
    if not isinstance(kinds, dict):
        return {}
    kind_config = kinds.get(kind)
    if not isinstance(kind_config, dict):
        return {}
    schema = kind_config.get("frontmatter_schema")
    return schema if isinstance(schema, dict) else {}


def universal_schema(root: Path, kinds: set[str]) -> dict[str, Any]:
    schema = _read_json(root / "schemas" / "_universal.schema.json")
    type_prop = schema.get("properties", {}).get("type")
    if isinstance(type_prop, dict) and isinstance(type_prop.get("enum"), list):
        type_prop["enum"] = sorted(set(type_prop["enum"]) | kinds)
    return schema


_SCALAR_TYPES = ("string", "integer", "number", "boolean", "null")
_ALL_JSON_TYPES = (*_SCALAR_TYPES, "array", "object")


def _widen_property_type(prop: dict[str, Any]) -> dict[str, Any]:
    """Widen a single property's ``type`` so it accepts any JSON value.

    Phase 0's LLM proposer routinely labels numeric wiki fields as
    ``{"type": "string"}`` and richer wiki fields (lists of resources, nested
    cost objects) as scalars too — because it sees them surrounded by
    string-y context (table cells). The compile LLM then correctly extracts
    them as integers (`hit_points: 125`), arrays (`produces: ['stone',
    'iron']`), or objects (`cost: {wood: 30, gold: 1200}`). Validation rejects
    everything that isn't a string. On a fresh run this produced ~70%
    quarantine on the first pass; widening to scalar-only still left ~2%
    quarantined on objects/arrays.

    Compromise: any **scalar** declaration is treated as "accepts any JSON
    value." A property the proposer genuinely typed as ``object`` keeps that
    constraint (the proposer was confident about shape); a scalar is treated
    as advisory. This keeps the universal schema's ``required`` list as the
    real presence gate while letting per-kind types stop being a quarantine
    source for LLM-proposed schemas.

    Engine- and game-agnostic by construction: the rule operates on schema
    keywords only, not field names or values. Recurses into ``items`` and
    nested object ``properties`` so the same rule applies at every depth.
    """
    if not isinstance(prop, dict):
        return prop
    widened = dict(prop)
    t = widened.get("type")
    if isinstance(t, str) and t in _SCALAR_TYPES:
        widened["type"] = list(_ALL_JSON_TYPES)
    if isinstance(widened.get("items"), dict):
        widened["items"] = _widen_property_type(widened["items"])
    if isinstance(widened.get("properties"), dict):
        widened["properties"] = {
            k: _widen_property_type(v) for k, v in widened["properties"].items()
        }
    return widened


def kind_frontmatter_schema(game_config: dict[str, Any], kind: str) -> dict[str, Any] | None:
    """Per-kind schema for VALIDATION; only enforces property types, not presence."""
    schema = raw_kind_schema(game_config, kind)
    properties = schema.get("properties") if isinstance(schema, dict) else None
    if not isinstance(properties, dict) or not properties:
        return None
    widened_properties = {k: _widen_property_type(v) for k, v in properties.items()}
    return {"type": "object", "properties": widened_properties}


def collect_required(schema: dict[str, Any]) -> list[str]:
    required = list(schema.get("required", []))
    for part in schema.get("allOf", []):
        if isinstance(part, dict):
            required.extend(part.get("required", []))
    return required


def validate_basic(data: dict[str, Any], schemas: list[dict[str, Any]]) -> list[str]:
    errors: list[str] = []
    for schema in schemas:
        for key in collect_required(schema):
            if key not in data:
                errors.append(f"required field missing: {key}")
    if "confidence" in data and not isinstance(data["confidence"], (int, float)):
        errors.append("confidence must be a number")
    return errors


def schema_registry(root: Path) -> Any | None:
    if Registry is None or Resource is None or DRAFT202012 is None:
        return None

    registry = Registry()
    schema_dir = root / "schemas"
    if not schema_dir.exists():
        return registry

    for path in schema_dir.glob("*.schema.json"):
        schema = _read_json(path)
        uri = str(schema.get("$id") or path.as_uri())
        resource = Resource.from_contents(schema, default_specification=DRAFT202012)
        registry = registry.with_resource(uri, resource)
        registry = registry.with_resource(path.as_uri(), resource)
    return registry


def validate_jsonschema(
    data: dict[str, Any], schemas: list[dict[str, Any]], root: Path
) -> list[str]:
    if jsonschema is None:
        return validate_basic(data, schemas)
    registry = schema_registry(root)
    errors: list[str] = []
    for schema in schemas:
        validator = (
            jsonschema.Draft202012Validator(schema, registry=registry)
            if registry is not None
            else jsonschema.Draft202012Validator(schema)
        )
        errors.extend(sorted(e.message for e in validator.iter_errors(data)))
    return errors


def validation_errors(
    root: Path,
    data: dict[str, Any],
    kind: str,
    kinds: set[str],
    game_config: dict[str, Any],
) -> tuple[list[str], bool]:
    schemas = [universal_schema(root, kinds)]
    kind_schema = kind_frontmatter_schema(game_config, kind)
    has_kind_schema = kind_schema is not None
    if kind_schema is not None:
        schemas.append(kind_schema)
    return validate_jsonschema(data, schemas, root), has_kind_schema
