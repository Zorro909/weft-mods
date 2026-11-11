// node shot.mjs <port> <model file> <out.png> [texture.png]
//
// Writes a picture of the model as Blockbench's main view shows it, with its textures on.

import { writeFileSync } from 'node:fs';
import path from 'node:path';
import { withBlockbench } from './devtools.mjs';
import { codecOf, openModel } from './models.mjs';

const [port, modelFile, pictureFile, namedTexture] = process.argv.slice(2);

await withBlockbench(port, async (blockbench) => {
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
    const view = await blockbench.evaluate(frameModel);
    writeFileSync(pictureFile, await blockbench.screenshot(view));
    console.log(pictureFile);
});

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

// Moves the camera along the direction a project opens with (from the north-west, above) until
// the view just holds the ball around the model. Gives the view's place in the window, once it
// is drawn that way.
async function frameModel() {
    const preview = Preview.selected;
    const bounds = new THREE.Box3();
    Outliner.elements.forEach((element) => bounds.expandByObject(element.mesh));
    if (!bounds.isEmpty()) {
        const ball = bounds.getBoundingSphere(new THREE.Sphere());
        const camera = preview.camPers;
        const halfAngleUp = THREE.MathUtils.degToRad(camera.fov) / 2;
        const halfAngleAcross = Math.atan(Math.tan(halfAngleUp) * camera.aspect);
        const distance = ball.radius / Math.sin(Math.min(halfAngleUp, halfAngleAcross));
        const towardsCamera = camera.position.clone().sub(preview.controls.target).normalize();
        preview.controls.target.copy(ball.center);
        camera.position.copy(ball.center).addScaledVector(towardsCamera, distance);
    }
    await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
    const { x, y, width, height } = preview.canvas.getBoundingClientRect();
    return { x, y, width, height };
}
