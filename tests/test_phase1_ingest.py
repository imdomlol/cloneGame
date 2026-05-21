import unittest
import warnings
from pathlib import Path
from tempfile import TemporaryDirectory

from scripts.phase1_ingest import (
    completed_source_for_kind,
    completed_source_index,
    completed_sources_for_other_kinds,
    frontmatter,
    migrate_existing_note,
    repair_frontmatter_delimiter,
    replace_frontmatter_type,
    source_key,
    trim_wikitext,
    validate_jsonschema,
)


class TrimWikitextTests(unittest.TestCase):
    def test_removes_html_comments(self) -> None:
        text = "Start <!-- hidden\ncomment --> End"
        self.assertEqual(trim_wikitext(text), "Start  End")

    def test_removes_category_links(self) -> None:
        text = "Alpha [[Category:Buildings]] Beta"
        self.assertEqual(trim_wikitext(text), "Alpha  Beta")

    def test_removes_file_with_nested_caption_links(self) -> None:
        text = "A [[File:Tower.png|thumb|See [[Tesla Tower]] now]] B"
        self.assertEqual(trim_wikitext(text), "A  B")

    def test_preserves_infobox_template_and_table(self) -> None:
        text = '{{Infobox building|hp=1200}}\n{| class="wikitable"\n|-\n| Cost || 40\n|}\n'
        self.assertEqual(trim_wikitext(text), text.strip())

    def test_preserves_bullets_formulas_and_links(self) -> None:
        text = "* Bullet\nDamage = atk * 1.5\nSee [[Ranger]]\n"
        self.assertEqual(trim_wikitext(text), text.strip())

    def test_removes_image_only_gallery(self) -> None:
        text = "<gallery>\nFile:A.png\n[[Image:B.png|thumb]]\n</gallery>\nAfter"
        self.assertEqual(trim_wikitext(text), "After")

    def test_preserves_gallery_with_prose(self) -> None:
        text = "<gallery>\nFile:A.png\nThis has context text.\n</gallery>"
        self.assertEqual(trim_wikitext(text), text)

    def test_collapses_repeated_blank_lines(self) -> None:
        text = "A\n\n\n\nB\n"
        self.assertEqual(trim_wikitext(text), "A\n\nB")


class FrontmatterTests(unittest.TestCase):
    def test_repairs_missing_closing_delimiter_before_first_heading(self) -> None:
        markdown = "---\nid: wood_workshop\nconfidence: 0.94\n## Description\nBody\n"

        repaired = repair_frontmatter_delimiter(markdown)

        self.assertEqual(
            repaired,
            "---\nid: wood_workshop\nconfidence: 0.94\n---\n## Description\nBody\n",
        )

    def test_frontmatter_parses_repaired_delimiter(self) -> None:
        markdown = "---\nid: wood_workshop\nconfidence: 0.94\n## Description\nBody\n"

        data, errors = frontmatter(markdown)

        self.assertEqual(errors, [])
        self.assertEqual(data["id"], "wood_workshop")
        self.assertEqual(data["confidence"], 0.94)

    def test_frontmatter_parses_single_dependency_flow_array(self) -> None:
        markdown = "---\nid: wood_tower\ndepends_on: [stone_tower]\n---\n"

        data, errors = frontmatter(markdown)

        self.assertEqual(errors, [])
        self.assertEqual(data["depends_on"], ["stone_tower"])

    def test_frontmatter_parses_multiple_dependency_flow_array(self) -> None:
        markdown = (
            "---\nid: the_lowlands\ndepends_on: [peaceful_lowlands, infected_executive]\n---\n"
        )

        data, errors = frontmatter(markdown)

        self.assertEqual(errors, [])
        self.assertEqual(data["depends_on"], ["peaceful_lowlands", "infected_executive"])

    def test_replace_frontmatter_type_updates_existing_type(self) -> None:
        markdown = "---\nid: infected\nname: Infected\ntype: infected\n---\nBody\n"

        updated = replace_frontmatter_type(markdown, "infected_unit")
        data, errors = frontmatter(updated)

        self.assertEqual(errors, [])
        self.assertEqual(data["type"], "infected_unit")
        self.assertIn("Body", updated)

    def test_replace_frontmatter_type_adds_missing_type(self) -> None:
        markdown = "---\nid: infected\nname: Infected\n---\nBody\n"

        updated = replace_frontmatter_type(markdown, "infected_unit")
        data, errors = frontmatter(updated)

        self.assertEqual(errors, [])
        self.assertEqual(data["type"], "infected_unit")


class SourceIndexTests(unittest.TestCase):
    def test_source_key_normalizes_encoding_case_and_trailing_slash(self) -> None:
        self.assertEqual(
            source_key("HTTPS://they-are-billions.fandom.com/wiki/Technology%20Tree/"),
            "https://they-are-billions.fandom.com/wiki/Technology_Tree",
        )

    def test_completed_source_index_excludes_quarantine(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            mechanic_dir = root / "vault" / "mechanic"
            quarantine_dir = root / "vault" / "_quarantine"
            mechanic_dir.mkdir(parents=True)
            quarantine_dir.mkdir(parents=True)
            completed = mechanic_dir / "technology_tree.md"
            completed.write_text(
                "---\n"
                "id: technology_tree\n"
                "source_url: https://they-are-billions.fandom.com/wiki/Technology_Tree\n"
                "---\n",
                encoding="utf-8",
            )
            quarantined = quarantine_dir / "duplicate.md"
            quarantined.write_text(
                "---\n"
                "id: duplicate\n"
                "source_url: https://they-are-billions.fandom.com/wiki/Duplicate\n"
                "---\n",
                encoding="utf-8",
            )

            index = completed_source_index(root)

            self.assertEqual(
                index["https://they-are-billions.fandom.com/wiki/Technology_Tree"],
                [("mechanic", completed)],
            )
            self.assertNotIn(
                "https://they-are-billions.fandom.com/wiki/Duplicate",
                index,
            )

    def test_source_index_matches_same_kind_only(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            mechanic_dir = root / "vault" / "mechanic"
            mechanic_dir.mkdir(parents=True)
            completed = mechanic_dir / "technology_tree.md"
            completed.write_text(
                "---\n"
                "id: technology_tree\n"
                "type: mechanic\n"
                "source_url: https://they-are-billions.fandom.com/wiki/Technology_Tree\n"
                "---\n",
                encoding="utf-8",
            )

            index = completed_source_index(root)
            source_url = "https://they-are-billions.fandom.com/wiki/Technology_Tree"

            self.assertEqual(completed_source_for_kind(index, source_url, "mechanic"), completed)
            self.assertIsNone(completed_source_for_kind(index, source_url, "research"))
            self.assertEqual(
                completed_sources_for_other_kinds(index, source_url, "research"),
                [("mechanic", completed)],
            )

    def test_migrate_existing_note_copies_note_to_new_kind_with_updated_type(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            infected_dir = root / "vault" / "infected"
            infected_dir.mkdir(parents=True)
            completed = infected_dir / "infected.md"
            source_url = "https://they-are-billions.fandom.com/wiki/Infected"
            completed.write_text(
                "---\n"
                "id: infected\n"
                "name: Infected\n"
                "type: infected\n"
                f"source_url: {source_url}\n"
                "---\n"
                "Body\n",
                encoding="utf-8",
            )
            index = completed_source_index(root)

            migrated, errors = migrate_existing_note(
                root, index, source_url, "infected_unit", completed
            )

            self.assertEqual(errors, [])
            self.assertEqual(migrated, root / "vault" / "infected_unit" / "infected.md")
            self.assertTrue(completed.exists())
            data, fm_errors = frontmatter(migrated.read_text(encoding="utf-8"))
            self.assertEqual(fm_errors, [])
            self.assertEqual(data["type"], "infected_unit")
            self.assertEqual(completed_source_for_kind(index, source_url, "infected_unit"), migrated)


class ValidationTests(unittest.TestCase):
    def test_validate_jsonschema_does_not_emit_refresolver_warning(self) -> None:
        schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object",
            "properties": {"id": {"type": "string"}},
        }

        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "schemas").mkdir()
            with warnings.catch_warnings():
                warnings.simplefilter("error", DeprecationWarning)
                errors = validate_jsonschema({"id": "technology_tree"}, [schema], root)

        self.assertEqual(errors, [])


if __name__ == "__main__":
    unittest.main()
