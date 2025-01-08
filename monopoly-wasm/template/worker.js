// Log console message to say we've loaded and started executing cirrectly
console.debug("Worker started");

// Load the WASM linkage
import init, { create_board, get_expected_frequencies } from "$link(prefix=./|../pkg/monopoly_wasm.js)";

import { perf_now, perf_end } from "$link(perf.js)";

// Debugging flag
let debug = false;

// Game board
let board;

// Executing flag
let executing = false;

// Execution target
let exec_target;

// Execution chunk size
let exec_chunk_size;

// Time to run for in milliseconds
const run_ms = 100;

// Initialise the WASM module
init({ module_or_path: new URL("$link(../pkg/monopoly_wasm_bg.wasm)", location.href) }).then(() => {
    // Set up message handler
    onmessage = (msg) => {
        const msgtype = msg.data.msgtype;
        switch (msgtype) {
            case "init":
            case "reinit":
                // Initialise debugging
                if (msgtype == "init") {
                    debug = msg.data.debug;
                }

                if (debug) {
                    console.debug(`Got ${msgtype} message`, msg.data)
                }

                // (Re-)initialise
                initialise(msg.data);

                break;

            case "calcexpected":
                // Calculate expected frequencies
                if (debug) {
                    console.debug("Got calcexpected message", msg.data)
                }

                calc_expected(msg.data);

                break;

            case "execstart":
                // Start execution
                if (debug) {
                    console.debug("Got execstart message", msg.data)
                }

                exec_start(msg.data);

                break;

            case "execstop":
                // Stop execution
                if (debug) {
                    console.debug("Got execstop message", msg.data)
                }

                exec_stop();

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
    // Stop executing if necessary
    exec_stop();

    // Set target to zero
    exec_target = 0;

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

function exec_stop() {
    executing = false;
}

function exec_start(msg) {
    // Set execution target
    exec_target = msg.target_turns;
    exec_chunk_size = msg.chunk_size;
    executing = true;
    setTimeout(exec_chunk, 0);
}

function exec_chunk() {
    // Start timer
    const start_time = perf_now();

    let cur_turns = board.get_turns();

    while (true) {
        // Calculate number of iterations to perform
        const turns_left = Number(BigInt(exec_target) - cur_turns);
        const turns = Math.min(turns_left, exec_chunk_size);

        let stop = true;

        if (turns > 0) {
            // Start timer
            const run_start_time = perf_now();

            // Run the game
            const rstats = board.run(turns);

            // Log performance
            perf_end(`chunkExec${rstats.turns}`, run_start_time);

            // Autopause?
            cur_turns = rstats.turns;

            if (cur_turns < exec_target) {
                stop = false;
            }

            // Send stats back
            postMessage({
                msgtype: "execfin",
                jailwait: rstats.jailwait,
                paused: stop,
                stats: build_jstats(rstats)
            });
        }

        // Need to stop?
        if (stop) {
            exec_stop();
            break;
        }

        // Use up time slot?
        const time_now = perf_now();

        if (time_now - start_time >= run_ms) {
            break;
        }
    }

    if (executing) {
        setTimeout(exec_chunk, 0);
    }

    // Log performance
    perf_end("intervalCallback", start_time);
}

function build_jstats(rstats) {
    // Chop reasons array
    const rreasons = rstats.reasons;
    const reasons = [];

    for (let i = 0; i < rreasons.length; i += rstats.reasons_stride) {
        reasons.push(rreasons.subarray(i, i + rstats.reasons_stride));
    }

    return {
        turns: rstats.turns,
        moves: rstats.moves,
        doubles: rstats.doubles,
        rollfreq: rstats.rollfreq,
        arrivals: rstats.arrivals,
        reasons: reasons,
    }
}

function calc_expected(msg) {
    // Mark start of run
    let start_time = perf_now();

    const freqs = get_expected_frequencies(msg.jailwait);

    // Mark end of run and measure
    const elapsed = perf_end("calcExpected", start_time);

    postMessage({
        msgtype: "calcexpectedfin",
        freq: freqs,
        duration: elapsed.duration,
    });
}