import init, { create_board } from "$link(prefix=./|../pkg/monopoly_wasm.js)";

let board;

init({ module_or_path: new URL("$link(../pkg/monopoly_wasm_bg.wasm)", location.href) }).then(() => {
    // Create the board
    board = create_board();

    // Set up message handler
    onmessage = (msg) => {
        switch (msg.data.msgtype) {
            case "execute":
                // Execute
                board.run(msg.data.ticks);

                // Send stats back
                postMessage({ msgtype: "execfin", stats: get_stats() });

                break;

            default:
                // Invalid message type
                console.error("Invalid message in worker:", msg);

                break;

        }
    };

    // Send ready message to main thread
    postMessage({ msgtype: "ready", square_desc: board.get_squares_desc(), square_type: board.get_squares_type() });

}).catch((e) => {
    console.error("Caught error in worker:", e)

});

function get_stats() {
    let rstats = board.get_stats();
    let doubles = board.get_doubles();
    let arrivals = board.get_arrivals();

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
    }
}