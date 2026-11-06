"""Tests for `readme.py`. Run: `python3 -m unittest discover -s tools/ci`."""

import unittest

from readme import parse, section

PINNED = "5faf2e3e57697dd3476b03a300f1231b84f6419e"


def release(tag: str, published: str, body: str = "") -> dict:
    return {"tag": tag, "url": f"https://r/{tag}", "date": published, "body": body}


def mod_json(weft_tag: str, commit: str) -> dict:
    return {"weft": {"tag": weft_tag, "commit": commit, "abi": "0123456789abcdef"}}


def table_row(readme: str, mod: str) -> str:
    return next(line for line in readme.splitlines() if line.startswith(f"| {mod} |"))


class Parse(unittest.TestCase):
    def test_candidate_of_weft_mod_action(self):
        parsed = parse(release("cogwork-v0.2.4-rc.61.gabc1234", "2026-11-07T00:00:00Z"), None)
        self.assertEqual((parsed.version, parsed.kind), ("0.2.4-rc.61.gabc1234", "rc"))

    def test_candidate(self):
        parsed = parse(release("cogwork-v0.1.1-rc.2", "2026-11-03T00:00:00Z"), None)
        self.assertEqual((parsed.version, parsed.kind), ("0.1.1-rc.2", "rc"))

    def test_full_release(self):
        parsed = parse(release("cogwork-v0.1.1", "2026-11-03T00:00:00Z"), None)
        self.assertEqual((parsed.version, parsed.kind), ("0.1.1", "release"))

    def test_experimental_build_is_left_out(self):
        self.assertIsNone(parse(release("cogwork-v0.1.0-experimental.ff9c712.8da6d93", "2026-11-02T00:00:00Z"), None))

    def test_weft_from_mod_json(self):
        parsed = parse(release("cogwork-v0.2.4", "2026-11-07T00:00:00Z"), mod_json("v26.11.2", PINNED))
        self.assertEqual(parsed.weft, f"26.11.2 [5faf2e3](https://github.com/Weft-Rust/Weft/commit/{PINNED})")

    def test_weft_from_the_notes_of_an_older_release(self):
        body = f"Built against Weft 26.10.4, commit [5faf2e3](https://x/commit/{PINNED}).\n"
        parsed = parse(release("cogwork-v0.1.0", "2026-11-02T00:00:00Z", body), None)
        self.assertEqual(parsed.weft, f"26.10.4 [5faf2e3](https://x/commit/{PINNED})")


class LatestTable(unittest.TestCase):
    def test_candidate_older_than_the_release_is_left_out(self):
        releases = [
            parse(release("cogwork-v0.1.0-rc.1", "2026-11-01T00:00:00Z"), None),
            parse(release("cogwork-v0.2.0", "2026-11-02T00:00:00Z"), None),
        ]
        row = table_row(section(releases), "cogwork")
        self.assertNotIn("rc.1", row, "a candidate before the release is stale")

    def test_candidate_after_the_release_is_listed(self):
        releases = [
            parse(release("cogwork-v0.2.0", "2026-11-02T00:00:00Z"), None),
            parse(release("cogwork-v0.2.1-rc.61.gabc1234", "2026-11-07T00:00:00Z"), None),
        ]
        row = table_row(section(releases), "cogwork")
        self.assertIn("0.2.1-rc.61.gabc1234", row)


if __name__ == "__main__":
    unittest.main()
