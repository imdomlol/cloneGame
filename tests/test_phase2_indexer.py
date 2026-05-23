"""Unit tests for phase2.indexer parsing helpers.

Covers the pure-Python helpers only: XML splitting, section splitting,
record building, and graph construction. Chroma upsert is out of scope
because it would require chromadb + sentence-transformers in CI.
"""

from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from phase2.indexer import (
    build_graph,
    build_records,
    deduplicate_records,
    parse_repomix,
    split_sections,
)

_SAMPLE_NOTE = """\
---
id: ranger
name: Ranger
type: unit
subtype: ranged
depends_on:
  - bow
  - soldier
---

## Description
The [[ranger]] is a ranged unit derived from the [[soldier]].

## Behavioral Mechanics
- IF target has tag:undead THEN damage *= 1.5
- ON hit: apply [[status_bleed]] for 3s

## References
- Source: https://example/wiki/Ranger
"""

_SAMPLE_NOTE_TWO = """\
---
id: bow
name: Bow
type: item
depends_on: []
---

## Description
A simple bow.

## Behavioral Mechanics
- IF drawn THEN fires arrow.

## References
- Source: https://example/wiki/Bow
"""


def _wrap_xml(*notes: tuple[str, str]) -> str:
    """Wrap (path, body) pairs in a minimal repomix-style XML envelope."""
    blocks = [f'<file path="{p}">\n{b}</file>' for p, b in notes]
    return "<files>\n" + "\n\n".join(blocks) + "\n</files>\n"


class ParseRepomixTests(unittest.TestCase):
    def test_extracts_path_and_body(self) -> None:
        xml = _wrap_xml(("vault/unit/ranger.md", _SAMPLE_NOTE))
        result = parse_repomix(xml)
        self.assertEqual(len(result), 1)
        path, body = result[0]
        self.assertEqual(path, "vault/unit/ranger.md")
        self.assertTrue(body.startswith("---\nid: ranger"))
        self.assertIn("## Behavioral Mechanics", body)

    def test_extracts_multiple_files(self) -> None:
        xml = _wrap_xml(
            ("vault/unit/ranger.md", _SAMPLE_NOTE),
            ("vault/item/bow.md", _SAMPLE_NOTE_TWO),
        )
        result = parse_repomix(xml)
        self.assertEqual([p for p, _ in result], ["vault/unit/ranger.md", "vault/item/bow.md"])

    def test_returns_empty_on_no_file_blocks(self) -> None:
        self.assertEqual(parse_repomix("<files>\n</files>\n"), [])

    def test_handles_inline_angle_brackets_in_body(self) -> None:
        body = _SAMPLE_NOTE.replace("simple bow", "<gallery>img</gallery> inline")
        xml = _wrap_xml(("vault/x/y.md", body))
        result = parse_repomix(xml)
        self.assertEqual(len(result), 1)


class SplitSectionsTests(unittest.TestCase):
    def test_extracts_three_sections(self) -> None:
        sections = split_sections(_SAMPLE_NOTE)
        self.assertIn("ranged unit", sections["description"])
        self.assertIn("tag:undead", sections["mechanics"])
        self.assertIn("https://example/wiki/Ranger", sections["references"])

    def test_strips_frontmatter_before_split(self) -> None:
        sections = split_sections(_SAMPLE_NOTE)
        self.assertNotIn("id: ranger", sections["description"])

    def test_missing_sections_are_empty_strings(self) -> None:
        note = "---\nid: x\n---\n\n## Description\nOnly prose here.\n"
        sections = split_sections(note)
        self.assertEqual(sections["description"], "Only prose here.")
        self.assertEqual(sections["mechanics"], "")
        self.assertEqual(sections["references"], "")

    def test_alternative_heading_wording_routes_to_mechanics(self) -> None:
        note = "---\nid: x\n---\n\n## Mechanics\n- IF foo THEN bar\n"
        self.assertIn("IF foo", split_sections(note)["mechanics"])


class BuildRecordsTests(unittest.TestCase):
    def _write(self, xml: str) -> Path:
        tmp = Path(self._tempdir.name) / "repomix.xml"
        tmp.write_text(xml, encoding="utf-8")
        return tmp

    def setUp(self) -> None:
        self._tempdir = TemporaryDirectory()
        self.addCleanup(self._tempdir.cleanup)

    def test_builds_one_record_per_valid_note(self) -> None:
        xml = _wrap_xml(
            ("vault/unit/ranger.md", _SAMPLE_NOTE),
            ("vault/item/bow.md", _SAMPLE_NOTE_TWO),
        )
        records = build_records(self._write(xml))
        self.assertEqual([r["id"] for r in records], ["ranger", "bow"])
        self.assertEqual(records[0]["type"], "unit")
        self.assertEqual(records[0]["subtype"], "ranged")
        self.assertIn("ranged unit", records[0]["description"])

    def test_skips_notes_without_frontmatter(self) -> None:
        xml = _wrap_xml(("vault/broken.md", "no frontmatter here\n## Description\nx\n"))
        self.assertEqual(build_records(self._write(xml)), [])

    def test_skips_notes_without_id(self) -> None:
        body = "---\nname: NoId\ntype: unit\n---\n\n## Description\nx\n"
        xml = _wrap_xml(("vault/broken.md", body))
        self.assertEqual(build_records(self._write(xml)), [])


class BuildGraphTests(unittest.TestCase):
    def test_emits_sorted_dedup_adjacency(self) -> None:
        records = [
            {
                "id": "ranger",
                "frontmatter": {"depends_on": ["bow", "soldier", "bow"]},
            },
            {
                "id": "bow",
                "frontmatter": {"depends_on": []},
            },
        ]
        graph = build_graph(records)
        self.assertEqual(graph["ranger"], ["bow", "soldier"])
        self.assertEqual(graph["bow"], [])

    def test_handles_missing_depends_on(self) -> None:
        graph = build_graph([{"id": "x", "frontmatter": {}}])
        self.assertEqual(graph["x"], [])

    def test_ignores_non_string_dependencies(self) -> None:
        records = [{"id": "x", "frontmatter": {"depends_on": ["a", 7, None, "b"]}}]
        self.assertEqual(build_graph(records)["x"], ["a", "b"])


class DeduplicateRecordsTests(unittest.TestCase):
    def test_keeps_first_occurrence(self) -> None:
        records = [
            {"id": "x", "path": "a", "frontmatter": {}},
            {"id": "x", "path": "b", "frontmatter": {}},
            {"id": "y", "path": "c", "frontmatter": {}},
        ]
        kept = deduplicate_records(records)
        self.assertEqual([r["path"] for r in kept], ["a", "c"])

    def test_no_duplicates_returns_input(self) -> None:
        records = [{"id": "x", "path": "a", "frontmatter": {}}]
        self.assertEqual(deduplicate_records(records), records)


if __name__ == "__main__":
    unittest.main()
