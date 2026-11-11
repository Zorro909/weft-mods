// node check.mjs <port> <compare|write> <model file>...
//
// Compares each file with the text Blockbench saves for it, and with `write` replaces the files
// that differ. Ends with 1 when files differ and stay so.

import { readFileSync, writeFileSync } from 'node:fs';
import { withBlockbench } from './devtools.mjs';
import { closeModel, codecOf, openModel } from './models.mjs';

const [port, mode, ...files] = process.argv.slice(2);
const rewrites = mode === 'write';

await withBlockbench(port, async (blockbench) => {
    // A file that is no model stops the run before any other is touched.
    files.forEach((file) => codecOf(file));
    const differing = await countDiffering(blockbench, files);
    console.log(`${files.length} checked, ${differing} ${rewrites ? 'written' : 'differ'}`);
    if (differing > 0 && !rewrites) process.exitCode = 1;
});

// Checks the files one after the other, as Blockbench holds one project at a time, and gives how
// many differ from what it saves.
async function countDiffering(blockbench, files) {
    if (files.length === 0) return 0;
    const [file, ...later] = files;
    const differs = await checkFile(blockbench, file);
    return (differs ? 1 : 0) + (await countDiffering(blockbench, later));
}

// Says whether the file differs from what Blockbench saves for it, and rewrites it if asked.
async function checkFile(blockbench, file) {
    const saved = Buffer.from(await savedText(blockbench, file));
    if (saved.equals(readFileSync(file))) {
        console.log(`same     ${file}`);
        return false;
    }
    if (rewrites) writeFileSync(file, saved);
    console.log(`${rewrites ? 'written' : 'differs'}  ${file}`);
    return true;
}

// What Blockbench saves for the model in this file. Blockbench ends a file without a newline;
// the files in the repository end with one, so it is added here.
async function savedText(blockbench, file) {
    await openModel(blockbench, file);
    const saved = await blockbench.evaluate((codec) => Codecs[codec].compile(), codecOf(file));
    await closeModel(blockbench);
    return `${saved}\n`;
}
