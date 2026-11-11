// The way into a running Blockbench: its window is a Chromium page, reached through the DevTools
// protocol on the debugging port it was started with.

const START_LIMIT_SECONDS = 30;

// Connects to the Blockbench behind `port`, hands it to `job` and lets go of it again. When
// anything fails, the process ends with 2 and the reason.
export async function withBlockbench(port, job) {
    try {
        const blockbench = await connect(port);
        await job(blockbench);
        blockbench.close();
    } catch (error) {
        console.error(`blockbench.sh: ${error.message}`);
        process.exit(2);
    }
}

async function connect(port) {
    const deadline = Date.now() + START_LIMIT_SECONDS * 1000;
    const socket = new WebSocket(await pageSocketUrl(port, deadline));
    await new Promise((resolve, reject) => {
        socket.onopen = resolve;
        socket.onerror = () => reject(new Error('Blockbench refused the DevTools connection'));
    });
    const blockbench = new BlockbenchPage(socket);
    await blockbench.waitUntilSetUp(deadline);
    return blockbench;
}

// The address of the window's page, which the port lists a moment after it opens.
async function pageSocketUrl(port, deadline) {
    while (Date.now() < deadline) {
        const targets = await (await fetch(`http://127.0.0.1:${port}/json`)).json();
        const page = targets.find((target) => target.type === 'page');
        if (page) return page.webSocketDebuggerUrl;
        await sleep(100);
    }
    throw new Error('Blockbench opened no window');
}

const sleep = (milliseconds) => new Promise((resolve) => setTimeout(resolve, milliseconds));

class BlockbenchPage {
    #socket;
    #lastId = 0;
    #callers = new Map();

    constructor(socket) {
        this.#socket = socket;
        socket.onmessage = (event) => this.#answer(JSON.parse(event.data));
        socket.onclose = () => this.#failAll(new Error('Blockbench went away'));
    }

    // Calls `pageFunction` with `values` inside Blockbench and gives back what it returns, after
    // waiting for it when that is a promise. Only the function's text crosses over: there it
    // sees Blockbench's globals, and nothing of the module it was written in.
    async evaluate(pageFunction, ...values) {
        const reply = await this.#call('Runtime.evaluate', {
            expression: `(${pageFunction})(...${JSON.stringify(values)})`,
            awaitPromise: true,
            returnByValue: true,
        });
        if (reply.exceptionDetails) throw new Error(reasonOf(reply.exceptionDetails));
        return reply.result.value;
    }

    // A PNG of one `area` of the window, given as `{x, y, width, height}`.
    async screenshot(area) {
        const clip = { ...area, scale: 1 };
        const reply = await this.#call('Page.captureScreenshot', { format: 'png', clip });
        return Buffer.from(reply.data, 'base64');
    }

    async waitUntilSetUp(deadline) {
        const isSetUp = () =>
            typeof Blockbench !== 'undefined' && Blockbench.setup_successful === true;
        while (!(await this.evaluate(isSetUp))) {
            if (Date.now() > deadline) throw new Error('Blockbench did not finish starting');
            await sleep(100);
        }
    }

    close() {
        this.#socket.onclose = null;
        this.#socket.close();
    }

    #call(method, params) {
        const id = ++this.#lastId;
        this.#socket.send(JSON.stringify({ id, method, params }));
        return new Promise((resolve, reject) => this.#callers.set(id, { resolve, reject }));
    }

    #answer(message) {
        const caller = this.#callers.get(message.id);
        if (!caller) return;
        this.#callers.delete(message.id);
        if (message.error) caller.reject(new Error(`DevTools: ${message.error.message}`));
        else caller.resolve(message.result);
    }

    #failAll(error) {
        for (const caller of this.#callers.values()) caller.reject(error);
        this.#callers.clear();
    }
}

// What went wrong in the page, without the stack below it.
function reasonOf(exceptionDetails) {
    const description = exceptionDetails.exception?.description ?? exceptionDetails.text;
    return description.split('\n')[0].replace(/^Error: /, '');
}
