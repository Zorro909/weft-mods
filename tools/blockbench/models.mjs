// Model files, and opening one in Blockbench with the codec that reads and writes it.

import { readFileSync } from 'node:fs';
import path from 'node:path';

// The codec of a model file, by its name: `Codecs.bedrock` for an entity, `Codecs.java_block`
// for a block.
export function codecOf(file) {
    if (file.endsWith('.geo.json')) return 'bedrock';
    const folders = path.dirname(path.resolve(file)).split(path.sep);
    if (file.endsWith('.json') && folders.includes('block')) return 'java_block';
    throw new Error(
        `${file} is neither an entity model (<name>.geo.json) ` +
            'nor a block model (a .json under a block/ folder)',
    );
}

// Opens the file as Blockbench's current project.
export async function openModel(blockbench, file) {
    const model = parsed(file);
    try {
        await blockbench.evaluate(openInPage, codecOf(file), path.resolve(file), model);
    } catch (error) {
        throw new Error(`Blockbench can't open ${file}: ${error.message}`);
    }
}

function parsed(file) {
    try {
        return JSON.parse(readFileSync(file, 'utf8'));
    } catch (error) {
        throw new Error(`${file} is not JSON: ${error.message}`);
    }
}

// Closes the current project without the question about unsaved changes.
export async function closeModel(blockbench) {
    await blockbench.evaluate(async () => {
        Project.saved = true;
        await Project.close(true);
    });
}

// Runs inside Blockbench. `resolve_parent: false` opens a model that only names a parent as it
// is, which is otherwise a question to the user. Any other question (several geometries in one
// file, nothing Blockbench knows as a model) leaves an empty project behind its dialog, so it
// counts as not opened.
function openInPage(codec, file, model) {
    Codecs[codec].load(model, { path: file }, { resolve_parent: false });
    if (Dialog.open) throw new Error(`it asks instead (dialog "${Dialog.open.id}")`);
}
