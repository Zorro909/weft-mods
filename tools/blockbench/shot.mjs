// node shot.mjs <port> <model file> <out.png> <view>[,<view>...] [texture.png]
//
// Writes a picture of the model as Blockbench's main view shows it, with its textures on. One
// view goes into <out.png>; several go into <out>-<view>.png each, which are printed.

import { writeFileSync } from 'node:fs';
import path from 'node:path';
import { withBlockbench } from './devtools.mjs';
import { codecOf, openModel } from './models.mjs';

// Where the camera stands for each view, as quarter turns round the model from where a project
// opens (north-west of it, above) and whether it looks from below instead.
const VIEWS = {
    north_west: { quarterTurns: 0, below: false },
    south_east: { quarterTurns: 2, below: false },
    north_east_below: { quarterTurns: 1, below: true },
    south_west_below: { quarterTurns: 3, below: true },
};

const [port, modelFile, pictureFile, viewNames, namedTexture] = process.argv.slice(2);

await withBlockbench(port, async (blockbench) => {
    const views = viewNames.split(',');
    const unknown = views.find((view) => !VIEWS[view]);
    if (unknown) throw new Error(`no view ${unknown}; there are ${Object.keys(VIEWS).join(', ')}`);
    await openModel(blockbench, modelFile);
    if (codecOf(modelFile) === 'bedrock') {
        const texture = path.resolve(namedTexture ?? ownTextureOf(modelFile));
        await blockbench.evaluate(addEntityTexture, texture);
    } else {
        await blockbench.evaluate(linkBlockTextures, texturesFolderOf(modelFile));
    }
    for (const texture of await blockbench.evaluate(texturesNotFound)) {
        console.error(`blockbench.sh: no texture ${texture}: the picture is without it`);
    }
    await blockbench.evaluate(rememberOpeningCamera);
    for (const view of views) {
        const file = views.length === 1 ? pictureFile : pictureOf(pictureFile, view);
        const area = await blockbench.evaluate(frameModel, VIEWS[view]);
        writeFileSync(file, await blockbench.screenshot(area));
        console.log(file);
    }
});

// The file of one view among several: `<out>-<view>.png` beside `<out>.png`.
function pictureOf(pictureFile, view) {
    const { dir, name, ext } = path.parse(pictureFile);
    return path.join(dir, `${name}-${view}${ext}`);
}

// `<mod>/assets/textures`, beside the `models` folder the file is under. Blockbench looks for a
// Minecraft resource pack around a model (`assets/<namespace>/textures`); a Weft mod has no
// namespace folder, so Blockbench finds no texture by itself.
function texturesFolderOf(modelFile) {
    const file = path.resolve(modelFile);
    const modelsFolder = file.lastIndexOf(`${path.sep}models${path.sep}`);
    if (modelsFolder < 0) {
        throw new Error(`${modelFile} is under no models/ folder to have a textures/ beside`);
    }
    return path.join(file.slice(0, modelsFolder), 'textures');
}

// An entity model names no texture: unless one is named for it, it is the picture of its name.
function ownTextureOf(modelFile) {
    const name = path.basename(modelFile, '.geo.json');
    return path.join(texturesFolderOf(modelFile), 'entity', `${name}.png`);
}

// The functions below run inside Blockbench.

// A block model names its textures as `block/<name>`, which Blockbench keeps as folder and name.
function linkBlockTextures(texturesFolder) {
    for (const texture of Texture.all) {
        texture.fromPath(`${texturesFolder}/${texture.folder}/${texture.name}`);
    }
}

function addEntityTexture(file) {
    new Texture().fromPath(file).add(false);
}

// Waits until every texture is read or given up on, and gives the files of the latter.
async function texturesNotFound() {
    const settled = (image) =>
        image.complete ||
        new Promise((resolve) => {
            image.addEventListener('load', resolve, { once: true });
            image.addEventListener('error', resolve, { once: true });
        });
    await Promise.all(Texture.all.map((texture) => settled(texture.img)));
    return Texture.all.filter((texture) => texture.error).map((texture) => texture.path);
}

// Keeps the direction a project opens with, from the model to the camera, for `frameModel` to
// turn its views from.
function rememberOpeningCamera() {
    const preview = Preview.selected;
    window.openingDirection = preview.camPers.position
        .clone()
        .sub(preview.controls.target)
        .normalize();
}

// Moves the camera to the `view`'s side of the model, as far out as the view just holds the ball
// around the model. Gives the view's place in the window, once the model is drawn that way: on a
// busy machine Blockbench's first frames are empty, and a picture of one would look like a
// model without boxes.
async function frameModel(view) {
    const FRAMES_TO_WAIT = 300;
    const preview = Preview.selected;
    const bounds = new THREE.Box3();
    Outliner.elements.forEach((element) => bounds.expandByObject(element.mesh));
    if (bounds.isEmpty()) throw new Error('the model has no boxes to show');
    const ball = bounds.getBoundingSphere(new THREE.Sphere());
    const camera = preview.camPers;
    const halfAngleUp = THREE.MathUtils.degToRad(camera.fov) / 2;
    const halfAngleAcross = Math.atan(Math.tan(halfAngleUp) * camera.aspect);
    const distance = ball.radius / Math.sin(Math.min(halfAngleUp, halfAngleAcross));
    const towardsCamera = window.openingDirection
        .clone()
        .applyAxisAngle(new THREE.Vector3(0, 1, 0), (-view.quarterTurns * Math.PI) / 2);
    if (view.below) towardsCamera.y = -towardsCamera.y;
    preview.controls.target.copy(ball.center);
    camera.position.copy(ball.center).addScaledVector(towardsCamera, distance);

    // Whether the view shows more than its background: read back right after a render, which
    // is the only moment a WebGL canvas still holds its picture.
    const probe = document.createElement('canvas');
    probe.width = probe.height = 32;
    const isDrawn = () => {
        preview.render();
        const context = probe.getContext('2d', { willReadFrequently: true });
        context.drawImage(preview.canvas, 0, 0, probe.width, probe.height);
        const pixels = new Uint32Array(context.getImageData(0, 0, 32, 32).data.buffer);
        return new Set(pixels).size > 8;
    };
    for (let frame = 0; frame < FRAMES_TO_WAIT; frame += 1) {
        await new Promise((resolve) => requestAnimationFrame(resolve));
        if (frame > 1 && isDrawn()) {
            const { x, y, width, height } = preview.canvas.getBoundingClientRect();
            return { x, y, width, height };
        }
    }
    throw new Error('Blockbench drew nothing of the model');
}
