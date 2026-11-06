"""Tests for `mod_json.py`. Run: `python3 -m unittest discover -s tools/ci`."""

import hashlib
import os
import tempfile
import unittest

from mod_json import abi_hash, mod_json, parse_deps, summarize, weft_tag

RELEASE = "ff9c7128511fc0dce7719b67c9a59274d70bdd5a"
MASTER = "4563878b067425085701268f074f9483b2625e4c"
TAGS = {"v26.11.1": RELEASE, "v26.10.4": "34fa22b1b6546aa8a8d3d12252bd7a5b01dbb0ec"}


class AbiHash(unittest.TestCase):
    def test_fnv1a_64_vectors(self):
        self.assertEqual(abi_hash(b""), "cbf29ce484222325")
        self.assertEqual(abi_hash(b"a"), "af63dc4c8601ec8c")
        self.assertEqual(abi_hash(b"foobar"), "85944171f73967e8")


class WeftTag(unittest.TestCase):
    def test_a_release(self):
        self.assertEqual(weft_tag(RELEASE, TAGS, lambda tag, commit: None), "v26.11.1")

    def test_a_commit_after_a_release_is_named_after_the_nearest(self):
        ahead = {"v26.11.1": 46, "v26.10.4": 102}
        self.assertEqual(weft_tag(MASTER, TAGS, lambda tag, commit: ahead[tag]), "v26.11.1-46-g4563878")

    def test_a_commit_after_no_release(self):
        with self.assertRaises(ValueError):
            weft_tag(MASTER, TAGS, lambda tag, commit: None)


class Summary(unittest.TestCase):
    def test_first_sentence_without_markup(self):
        description = "Plays films: {#a0a0a0}/showcase play{/} plays one. More.\\n\\nSecond paragraph."
        self.assertEqual(summarize(description), "Plays films: /showcase play plays one.")

    def test_long_sentence_is_cut_at_a_comma(self):
        description = (
            "Gears, item tubes and engines, an excavator that digs out a whole area layer by layer, an "
            "assembly bench that crafts on its own and a leveler that makes land flat."
        )
        summary = summarize(description)
        self.assertLessEqual(len(summary), 140)
        self.assertEqual(summary, "Gears, item tubes and engines, an excavator that digs out a whole area layer by layer…")


class Deps(unittest.TestCase):
    def test_the_release_build_has_no_features(self):
        lib_rs = (
            '#[cfg(not(feature = "showcase"))]\ndeclare_mod!(deps: ["base"], init: init);\n'
            '#[cfg(feature = "showcase")]\ndeclare_mod!(deps: ["base", "showcase"], init: init);\n'
        )
        self.assertEqual(parse_deps(lib_rs), ["base"])

    def test_no_deps(self):
        self.assertEqual(parse_deps("declare_mod!(deps: [], init: init);"), [])


class Record(unittest.TestCase):
    def setUp(self):
        self.cwd = os.getcwd()
        self.tmp = tempfile.TemporaryDirectory()
        os.chdir(self.tmp.name)
        os.makedirs("cogwork/assets/lang")
        os.makedirs("cogwork/src")
        with open("cogwork/assets/lang/en_us.lang", "w") as f:
            f.write("mod.cogwork.name = Cogwork\nmod.cogwork.description = Gears. More.\n")
        with open("cogwork/src/lib.rs", "w") as f:
            f.write('declare_mod!(deps: ["base"], init: init);\n')

    def tearDown(self):
        os.chdir(self.cwd)
        self.tmp.cleanup()

    def record(self, tag="cogwork-v0.2.0", kind="release"):
        weft = {"tag": "v26.11.1", "commit": RELEASE, "abi": "f470b1cbb1f9ad91"}
        return mod_json("cogwork", "cogwork", tag, kind, b"zip", "Zorro909/weft-mods", "c" * 40, weft,
                        "2026-11-04T12:00:00Z")

    def test_fields(self):
        record = self.record()
        self.assertEqual(record["schema"], 1)
        self.assertEqual((record["id"], record["name"], record["summary"]), ("cogwork", "Cogwork", "Gears."))
        self.assertEqual(record["authors"], ["Zorro909"])
        self.assertIsNone(record["icon"])
        self.assertEqual((record["version"], record["channel"]), ("0.2.0", "release"))
        self.assertEqual(record["deps"], [{"id": "base"}])
        self.assertEqual(record["source"], {"repo": "Zorro909/weft-mods", "commit": "c" * 40})
        self.assertEqual(record["asset"], {
            "name": "cogwork.zip",
            "url": "https://github.com/Zorro909/weft-mods/releases/download/cogwork-v0.2.0/cogwork.zip",
            "size": 3,
            "sha256": hashlib.sha256(b"zip").hexdigest(),
        })
        self.assertNotIn("reviewed", record)

    def test_an_experimental_version_is_its_tag(self):
        record = self.record("cogwork-v0.2.0-experimental.4563878.abc1234", "experimental")
        self.assertEqual((record["version"], record["channel"]), ("0.2.0-experimental.4563878.abc1234", "experimental"))

    def test_a_candidate_is_a_prerelease(self):
        self.assertEqual(self.record("cogwork-v0.2.1-rc.1", "rc")["channel"], "prerelease")

    def test_icon_at_the_source_commit(self):
        os.makedirs("cogwork/assets/textures/gui")
        open("cogwork/assets/textures/gui/icon.png", "wb").close()
        self.assertEqual(
            self.record()["icon"],
            f"https://raw.githubusercontent.com/Zorro909/weft-mods/{'c' * 40}/cogwork/assets/textures/gui/icon.png",
        )


if __name__ == "__main__":
    unittest.main()
