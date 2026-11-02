"""Tests for `readme.py`. Run: `python3 -m unittest discover -s tools/ci`."""

import unittest

from readme import parse, section

PINNED = "5faf2e3e57697dd3476b03a300f1231b84f6419e"
MASTER = "ff9c7128511fc0dce7719b67c9a59274d70bdd5a"


def release(tag: str, published: str, weft: str) -> dict:
    body = f"Built against Weft 26.10.4, commit [{weft[:7]}](https://x/commit/{weft}).\n"
    return {"tag": tag, "url": f"https://r/{tag}", "date": published, "body": body}


def table_row(readme: str, mod: str) -> str:
    return next(line for line in readme.splitlines() if line.startswith(f"| {mod} |"))


def contains_nothing(older: str, newer: str) -> bool:
    return False


class LatestTable(unittest.TestCase):
    def test_experimental_published_before_the_release_is_listed(self):
        releases = [
            parse(release("cogwork-v0.1.0-experimental.ff9c712.8da6d93", "2026-11-02T00:54:35Z", MASTER)),
            parse(release("cogwork-v0.1.0", "2026-11-02T00:54:53Z", PINNED)),
        ]
        row = table_row(section(releases, contains_nothing), "cogwork")
        self.assertIn("cogwork-v0.1.0-experimental", row, "an experimental build on a newer Weft is listed")

    def test_experimental_on_a_weft_the_release_has_is_left_out(self):
        releases = [
            parse(release("cogwork-v0.1.0-experimental.5faf2e3.8da6d93", "2026-11-01T00:00:00Z", PINNED)),
            parse(release("cogwork-v0.2.0", "2026-11-02T00:00:00Z", MASTER)),
        ]
        contains = lambda older, newer: (older, newer) == (PINNED, MASTER)
        row = table_row(section(releases, contains), "cogwork")
        self.assertNotIn("experimental", row, "the release already has that Weft")

    def test_candidate_older_than_the_release_is_left_out(self):
        releases = [
            parse(release("cogwork-v0.1.0-rc.1", "2026-11-01T00:00:00Z", PINNED)),
            parse(release("cogwork-v0.2.0", "2026-11-02T00:00:00Z", PINNED)),
        ]
        row = table_row(section(releases, contains_nothing), "cogwork")
        self.assertNotIn("rc.1", row, "a candidate before the release is stale")


if __name__ == "__main__":
    unittest.main()
