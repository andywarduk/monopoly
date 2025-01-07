// Log console message to say we've loaded and started executing cirrectly
console.debug("Worker started");

// Load the WASM linkage
import init, { create_board, get_expected_frequencies } from "$link(prefix=./|../pkg/monopoly_wasm.js)";

// Debugging flag
let debug = false;

// Game board
let board;

// Initialise the WASM module
init({ module_or_path: new URL("$link(../pkg/monopoly_wasm_bg.wasm)", location.href) }).then(() => {
    // Set up message handler
    onmessage = (msg) => {
        switch (msg.data.msgtype) {
            case "init":
            case "reinit":
                // Initialise
                if (msg.data.msgtype == "init") {
                    debug = msg.data.debug;
                }

                if (debug) {
                    console.debug(`Got ${msg.data.msgtype} message`, msg.data)
                }

                initialise(msg.data);

                break;

            case "calcexpected":
                if (debug) {
                    console.debug("Got calcexpected message", msg.data)
                }

                postMessage({
                    msgtype: "calcexpectedfin",
                    freq: get_expected_frequencies(msg.data.jailwait)
                });

                break;

            case "exec":
                // Execute
                if (debug) {
                    console.debug("Got exec message", msg.data)
                }

                const ticks = msg.data.ticks;
                const chunk = msg.data.chunk;

                // Mark start of run
                let start_mark = `runStart${chunk}`
                performance.mark(start_mark);

                // Run the game
                let rstats = board.run(ticks);

                // Mark end of run
                let end_mark = `runEnd${chunk}`
                performance.mark(end_mark);

                // Add measure
                const elapsed = performance.measure(
                    `run${chunk}`,
                    {
                        detail: { ticks: ticks },
                        start: start_mark,
                        end: end_mark,
                    }
                );

                // Send stats back
                postMessage({
                    msgtype: "execfin",
                    duration: elapsed.duration,
                    jailwait: rstats.jailwait,
                    stats: build_jstats(rstats, chunk),
                });

                break;

            default:
                // Invalid message type
                console.error("Invalid message in worker:", msg);

                break;

        }
    };

    // Send loaded message to main thread
    postMessage({ msgtype: "loaded" });

}).catch((e) => {
    console.error("Caught error in worker:", e)

});

function initialise(msg) {
    // Create the board
    board = create_board(msg.jailwait);

    // Send result to main thread
    let ret = {
        msgtype: `${msg.msgtype}fin`
    };

    if (msg.msgtype == "init") {
        // Send extra data back for first initialise
        ret.spaces = board.get_spaces();
        ret.arrival_reasons = board.get_arrival_reason_descs();
    }

    postMessage(ret);
}

function build_jstats(rstats, chunk) {
    // Chop reasons array
    const rreasons = rstats.reasons;
    const reasons = [];

    for (let i = 0; i < rreasons.length; i += rstats.reasons_stride) {
        reasons.push(rreasons.subarray(i, i + rstats.reasons_stride));
    }

    return {
        chunk: chunk,
        turns: rstats.turns,
        moves: rstats.moves,
        doubles: rstats.doubles,
        rollfreq: rstats.rollfreq,
        arrivals: rstats.arrivals,
        reasons: reasons,
    }
}
