let debug = false;
let paused = false;
const iterations = 1000;
let worker;
let square_desc;
let square_type;

setup();

function setup() {
    // Set up web worker
    if (window.Worker) {
        worker = new Worker("worker.js", { type: "module" });
        worker.onmessage = process_message;
    } else {
        alert("Web workers not available")
    }
}

function process_message(msg) {
    if (debug) {
        console.debug("Got message from worker:", msg);
    }

    switch (msg.data.msgtype) {
        case "ready":
            if (debug) {
                console.debug("Got 'ready' from worker:", msg.data);
            }

            // Process the message
            process_ready(msg.data);

            // Start worker running
            worker.postMessage({ msgtype: "execute", ticks: iterations });

            break;

        case "execfin":
            if (debug) {
                console.debug("Got 'execfin' from worker:", msg.data);
            }

            // Process the stats
            process_stats(msg.data.stats);

            if (!paused) {
                // Execute next
                worker.postMessage({ msgtype: "execute", ticks: iterations });
            }

            break;

        default:
            console.error("Invalid message from worker:", msg);
            break;

    }
}

function process_ready(data) {
    // Set up board squares
    square_desc = data.square_desc;
    square_type = data.square_type;

    for (const [index, desc] of square_desc.entries()) {
        // Get type
        let type = square_type[index];

        // Find square table cell
        let elem = document.getElementById(index.toString());

        // Create div
        let div = document.createElement("div");

        let orient;

        if (index == 0) {
            orient = "nw";
        } else if (index < 10) {
            orient = "n";
        } else if (index == 10) {
            orient = "ne";
        } else if (index < 20) {
            orient = "e";
        } else if (index == 20) {
            orient = "se";
        } else if (index < 30) {
            orient = "s";
        } else if (index == 30) {
            orient = "sw";
        } else {
            orient = "w";
        }

        div.setAttribute("class", `square_div square_${orient}`);

        if (type == 'P') {
            // Add colour block for property squares
            let colourdiv = document.createElement("div");
            colourdiv.setAttribute("class", "colour_block");

            // Work out colour
            let colour;

            switch (desc[0]) {
                case 'A':
                    colour = "rgb(140,87,60)";
                    break;
                case 'B':
                    colour = "rgb(181,223,249)";
                    break;
                case 'C':
                    colour = "rgb(200,71,147)";
                    break;
                case 'D':
                    colour = "rgb(233,153,63)";
                    break;
                case 'E':
                    colour = "rgb(218,56,51)";
                    break;
                case 'F':
                    colour = "rgb(252,243,80)";
                    break;
                case 'G':
                    colour = "rgb(85,176,99)";
                    break;
                case 'H':
                    colour = "rgb(48,112,182)";
                    break;
            }
            colourdiv.setAttribute("style", `background-color: ${colour};`);
            div.appendChild(colourdiv);
        }

        // Create description
        let descspan = document.createElement("p");
        descspan.setAttribute("class", "desc");
        descspan.innerText = desc;
        div.appendChild(descspan);

        // Create percentage span
        let pctspan = document.createElement("p");
        pctspan.setAttribute("id", `pct${index}`);
        div.appendChild(pctspan);

        // Add the div
        elem.appendChild(div);
    }

    setup_pause();

    let pause = document.getElementById("pause");
    pause.onclick = pause_click;
    pause.style.display = "block";
}

function process_stats(stats) {
    if (debug) {
        console.debug("Processing stats:", stats);
    }

    let elem;

    elem = document.getElementById("stat_turns");
    elem.innerText = stats.turns.toLocaleString();

    elem = document.getElementById("stat_moves");
    elem.innerText = stats.moves.toLocaleString();

    elem = document.getElementById("stat_throws");
    elem.innerText = stats.throws.toLocaleString();

    elem = document.getElementById("stat_doubles");
    elem.innerText = stats.doubles[0].toLocaleString();

    elem = document.getElementById("stat_doubles_pct");
    elem.innerText = percent(stats.doubles[0], stats.turns);

    elem = document.getElementById("stat_ddoubles");
    elem.innerText = stats.doubles[1].toLocaleString();

    elem = document.getElementById("stat_ddoubles_pct");
    elem.innerText = percent(stats.doubles[1], stats.turns);

    elem = document.getElementById("stat_tdoubles");
    elem.innerText = stats.doubles[2].toLocaleString();

    elem = document.getElementById("stat_tdoubles_pct");
    elem.innerText = percent(stats.doubles[2], stats.turns);

    let leaderboard = [];

    for (const [index, arrivals] of stats.arrivals.entries()) {
        let elem = document.getElementById(`pct${index}`);
        elem.innerText = percent(arrivals, stats.moves);

        leaderboard.push([index, arrivals]);
    };

    leaderboard.sort(([_ia, aa], [_ib, ab]) => Number(ab - aa));

    let container = document.getElementById("leaderboard");
    container.innerHTML = "";

    let table = document.createElement("table");
    container.appendChild(table);

    let tbody = document.createElement("tbody");
    table.appendChild(tbody);

    for (let i = 0; i < 10; i++) {
        let elem = leaderboard[i][0];
        let stat = leaderboard[i][1];

        add_stat(tbody, square_desc[elem], stat, stats.moves)

        let reasons = stats.reasons[elem];

        if (square_type[elem] == 'J') {
            let visits = stat - reasons.reduce((a, b) => a + b, 0n);
            add_stat(tbody, "Just Visiting", visits, stat, true)
        }

        for (const [index, count] of reasons.entries()) {
            if (count == 0) {
                continue;
            }

            let desc;

            switch (index) {
                case 0:
                    desc = "Chance Card";
                    break
                case 1:
                    desc = "Community Chest Card";
                    break
                case 2:
                    desc = "Go to Jail";
                    break
                case 3:
                    desc = "Triple Double";
                    break
            }

            add_stat(tbody, desc, count, stat, true)
        }
    }
}

function add_stat(tbody, desc, value, total, sub) {
    let tr = document.createElement("tr");

    if (sub) {
        tr.setAttribute("class", "substat");
    }

    tbody.appendChild(tr);

    let td = document.createElement("td");
    td.setAttribute("class", "statlabel");
    td.innerText = `${desc}:`;
    tr.appendChild(td);

    td = document.createElement("td");
    td.setAttribute("class", "stat");
    td.innerText = value.toLocaleString();
    tr.appendChild(td);

    td = document.createElement("td");
    td.setAttribute("class", "statpct");
    td.innerText = percent(value, total);
    tr.appendChild(td);
}

function percent(value, total) {
    if (total == 0) {
        return 0;
    }

    return (Number(value) / Number(total)).toLocaleString(undefined, { style: "percent", "minimumFractionDigits": 2, "maximumFractionDigits": 2 });
}

function pause_click() {
    paused = !paused;

    setup_pause();

    if (!paused) {
        worker.postMessage({ msgtype: "execute", ticks: iterations });
    }
}

function setup_pause() {
    let pause = document.getElementById("pause");

    if (!paused) {
        pause.innerText = "⏸︎ Pause";
    } else {
        pause.innerText = "▶ Play";
    }
}