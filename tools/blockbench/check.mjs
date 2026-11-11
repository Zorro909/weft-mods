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
    let differing = 0;
    for (const file of files) {
        const saved = Buffer.from(await savedText(blockbench, file));
        if (saved.equals(readFileSync(file))) {
            console.log(`same     ${file}`);
            continue;
        }
        differing += 1;
        if (rewrites) writeFileSync(file, saved);
        console.log(`${rewrites ? 'written' : 'differs'}  ${file}`);
    }
    console.log(`${files.length} checked, ${differing} ${rewrites ? 'written' : 'differ'}`);
    if (differing > 0 && !rewrites) process.exitCode = 1;
});

// What Blockbench saves for the model in this file. Blockbench ends a file without a newline;
// the files in the repository end with one, so it is added here.
async function savedText(blockbench, file) {
    await openModel(blockbench, file);
    const saved = await blockbench.evaluate((codec) => Codecs[codec].compile(), codecOf(file));
    await closeModel(blockbench);
    return `${saved}\n`;
}
