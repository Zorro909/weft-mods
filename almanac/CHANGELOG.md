# Changelog

## Unreleased

### Added

- The Almanac, a quest book opened with J or from the pause menu: a tab for every chapter, its
  quests in chains, and for the quest you click what it asks, how far you are and what it gives.
- Quests count what you do: items you hold, blocks you place and break, mobs you defeat, items
  you use, and whatever a quest's own mod reports.
- A note slides in when a quest is finished, and its rewards are claimed from the book.
- `almanac_api` for other mods: chapters and quests of their own, a page they lay out
  themselves, and changing or removing another mod's quests.
- `/almanac reset` and `/almanac finish <quest>`, and the condition `almanac:done(<quest>)`.
