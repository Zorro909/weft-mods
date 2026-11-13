"""Tests for the asset lint. Run: `python3 -m unittest discover -s tools/assets`."""

import json
import tempfile
import unittest
from pathlib import Path

import block_models
import entity_models
import lint
import textures

# The first 24 bytes of a PNG 64 wide and 32 high: all the lint reads of one.
PNG_64_BY_32 = textures.PNG_SIGNATURE + b"\x00\x00\x00\rIHDR" + (64).to_bytes(4) + (32).to_bytes(4)
ALL_SIDES = {side: {"uv": [0, 0, 16, 16]} for side in block_models.SIDES}


def box(start: list, end: list, **more) -> dict:
    return {"from": start, "to": end, "faces": ALL_SIDES, **more}


class Mod(unittest.TestCase):
    """A mod's `assets` folder in a temporary place, to write models and textures into."""

    def setUp(self):
        folder = tempfile.TemporaryDirectory()
        self.addCleanup(folder.cleanup)
        self.mod = Path(folder.name)
        self.assets = self.mod / "assets"

    def write(self, path: str, content) -> Path:
        file = self.assets / path
        file.parent.mkdir(parents=True, exist_ok=True)
        if isinstance(content, bytes):
            file.write_bytes(content)
        else:
            file.write_text(content if isinstance(content, str) else json.dumps(content))
        return file

    def block_faults(self, model: dict, path: str = "models/block/thing.json") -> list[str]:
        return block_models.faults(self.write(path, model), self.assets)

    def entity_faults(self, bones: list[dict], skin=(64, 32)) -> list[str]:
        description = {"texture_width": skin[0], "texture_height": skin[1]}
        model = {"minecraft:geometry": [{"description": description, "bones": bones}]}
        return entity_models.faults(self.write("models/thing.geo.json", model), self.assets)


class BlockModels(Mod):
    def test_boxes_apart_pass(self):
        model = {"elements": [box([0, 0, 0], [16, 4, 16]), box([4, 4, 4], [12, 16, 12])]}
        self.assertEqual(self.block_faults(model), [])

    def test_two_faces_in_one_plane_flicker(self):
        model = {"elements": [box([0, 0, 0], [16, 4, 16]), box([4, 0, 4], [12, 4, 12])]}
        found = self.block_faults(model)
        self.assertEqual(len(found), 2, "the up faces and the down faces")
        self.assertIn("the down faces of box 0 and box 1 share the plane at 0", found[0])

    def test_faces_in_one_plane_that_only_touch_pass(self):
        model = {"elements": [box([0, 0, 0], [8, 4, 16]), box([8, 0, 0], [16, 4, 16])]}
        self.assertEqual(self.block_faults(model), [])

    def test_a_face_left_out_does_not_flicker(self):
        inner = {"from": [4, 0, 4], "to": [12, 4, 12], "faces": {"north": {}}}
        self.assertEqual(self.block_faults({"elements": [box([0, 0, 0], [16, 4, 16]), inner]}), [])

    def test_a_corner_between_texels_is_named(self):
        found = self.block_faults({"elements": [box([0, 0, 0], [16, 4.5, 16], name="plate")]})
        self.assertEqual(len(found), 1)
        self.assertIn('box "plate" has corners between texels ([4.5])', found[0])

    def test_an_item_may_end_between_texels(self):
        model = {"elements": [box([0, 0, 0], [16, 4.5, 16])]}
        self.assertEqual(self.block_faults(model, "models/item/thing.json"), [])

    def test_a_turned_box_is_left_to_the_eye(self):
        turned = box([0, 0, 0], [16, 4.5, 16], rotation={"angle": 45, "axis": "y"})
        self.assertEqual(self.block_faults({"elements": [turned, box([0, 0, 0], [16, 4, 16])]}), [])

    def test_a_box_inside_out_is_named(self):
        found = self.block_faults({"elements": [box([8, 0, 0], [4, 4, 16])]})
        self.assertIn("goes from [8, 0, 0] back to [4, 4, 16]", found[0])

    def test_a_uv_past_the_texture_is_named(self):
        faces = {"up": {"uv": [0, 0, 20, 16]}}
        found = self.block_faults({"elements": [{"from": [0, 0, 0], "to": [1, 1, 1], "faces": faces}]})
        self.assertIn("the uv of its up face leaves the texture ([20])", found[0])

    def test_a_texture_without_its_file_is_named(self):
        self.write("textures/block/there.png", PNG_64_BY_32)
        named = {"a": "block/there", "b": "block/gone", "c": "#a", "d": "base:block/stone"}
        found = self.block_faults({"textures": named})
        self.assertEqual(found, ['texture "b" names block/gone, and there is no textures/block/gone.png'])

    def test_a_file_that_is_no_json_is_named(self):
        file = self.write("models/block/broken.json", "{")
        self.assertIn("not JSON", block_models.faults(file, self.assets)[0])


class EntityModels(Mod):
    def test_a_model_inside_its_texture_passes(self):
        bones = [{"name": "body", "cubes": [{"origin": [0, 0, 0], "size": [8, 12, 8], "uv": [0, 0]}]}]
        self.assertEqual(self.entity_faults(bones), [])

    def test_an_unwrap_past_the_texture_is_named(self):
        bones = [{"name": "body", "cubes": [{"size": [8, 12, 8], "uv": [40, 16]}]}]
        found = self.entity_faults(bones)
        self.assertIn('cube 0 of bone "body" unwraps to 72 x 36, past the texture\'s 64 x 32', found[0])

    def test_faces_with_a_uv_each_are_left_alone(self):
        bones = [{"name": "body", "cubes": [{"size": [80, 80, 80], "uv": {"north": {"uv": [0, 0]}}}]}]
        self.assertEqual(self.entity_faults(bones), [])

    def test_a_turned_cube_is_named(self):
        bones = [{"name": "arm", "cubes": [{"size": [1, 1, 1], "uv": [0, 0], "rotation": [0, 45, 0]}]}]
        self.assertIn("is turned, and the game draws it unturned", self.entity_faults(bones)[0])

    def test_a_child_before_its_parent_is_named(self):
        found = self.entity_faults([{"name": "tip", "parent": "bit"}, {"name": "bit"}])
        self.assertEqual(found, ['bone "tip" comes before its parent "bit", or has none'])

    def test_a_texture_of_another_shape_is_named(self):
        self.write("textures/entity/thing.png", PNG_64_BY_32)
        self.assertEqual(self.entity_faults([]), [])
        self.assertEqual(self.entity_faults([], skin=(32, 16)), [], "twice the size fits")
        self.assertIn("no whole multiple of the model's 64 x 64", self.entity_faults([], skin=(64, 64))[0])

    def test_a_file_of_two_geometries_is_named(self):
        file = self.write("models/two.geo.json", {"minecraft:geometry": [{}, {}]})
        self.assertIn("holds 2 geometries", entity_models.faults(file, self.assets)[0])

    def test_a_file_that_is_no_json_is_named(self):
        file = self.write("models/broken.geo.json", "{")
        self.assertIn("not JSON", entity_models.faults(file, self.assets)[0])


class Textures(Mod):
    def test_a_png_passes(self):
        self.assertEqual(textures.faults(self.write("textures/block/a.png", PNG_64_BY_32)), [])
        self.assertEqual(textures.size_of(self.assets / "textures/block/a.png"), (64, 32))

    def test_another_file_named_png_is_named(self):
        found = textures.faults(self.write("textures/block/a.png", b"GIF89a"))
        self.assertEqual(found, ["is no PNG file, whatever its name says"])


class Lint(Mod):
    def test_a_mod_without_faults_ends_with_0(self):
        self.write("models/block/thing.json", {"elements": [box([0, 0, 0], [16, 16, 16])]})
        self.assertEqual(lint.main([str(self.mod)]), 0)

    def test_every_fault_names_its_file(self):
        self.write("models/block/thing.json", {"elements": [box([0, 0, 0], [16, 4.5, 16])]})
        self.write("models/thing.geo.json", {"minecraft:geometry": []})
        self.write("textures/item/a.png", b"")
        found = lint.faults_of(self.mod)
        self.assertEqual(len(found), 3)
        self.assertTrue(found[0].startswith(f"{self.assets}/models/block/thing.json: "))
        self.assertEqual(lint.main([str(self.mod)]), 1)

    def test_no_folder_is_a_mistake(self):
        self.assertEqual(lint.main([]), 2)

    def test_a_mod_without_assets_passes(self):
        self.assertEqual(lint.faults_of(self.mod), [])


if __name__ == "__main__":
    unittest.main()
