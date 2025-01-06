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

                board.run(msg.data.ticks);

                // Send stats back
                postMessage({
                    msgtype: "execfin",
                    stats: get_stats()
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
        ret.spaces_desc = board.get_spaces_desc();
        ret.spaces_type = board.get_spaces_type();
        ret.arrival_reason_descs = board.get_arrival_reason_descs();
    }

    postMessage(ret);
}

function get_stats() {
    let rstats = board.get_stats();
    let doubles = board.get_doubles();
    let arrivals = board.get_arrivals();
    let rollfreq = board.get_rollfreq();

    let reasons = [];

    for (let i = 0; i < arrivals.length; i++) {
        reasons.push(board.get_arrival_reasons(i));
    }

    return {
        turns: rstats.turns,
        moves: rstats.moves,
        doubles: doubles,
        arrivals: arrivals,
        reasons: reasons,
        rollfreq: rollfreq,
        jailwait: rstats.jailwait,
    }
}