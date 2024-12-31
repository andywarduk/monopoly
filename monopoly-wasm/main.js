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

        // Calculate orientation
        let orient;
        let side;

        if (index == 0) {
            orient = "nw";
            side = "c";
        } else if (index < 10) {
            orient = "n";
            side = "tb";
        } else if (index == 10) {
            orient = "ne";
            side = "c";
        } else if (index < 20) {
            orient = "e";
            side = "lr";
        } else if (index == 20) {
            orient = "se";
            side = "c";
        } else if (index < 30) {
            orient = "s";
            side = "tb";
        } else if (index == 30) {
            orient = "sw";
            side = "c";
        } else {
            orient = "w";
            side = "lr";
        }

        // Create divs
        let reldiv = document.createElement("div");
        reldiv.setAttribute("class", `space_reldiv space_reldiv_${orient} space_reldiv_${side}`);

        let div = document.createElement("div");
        div.setAttribute("class", `space_div space_div_${orient} space_div_${side}`);

        if (type == 'P') {
            // Add colour block for property squares

            // Work out colour
            let colour = set_to_colour(desc[0]);

            // Add the div
            let colourdiv = document.createElement("div");

            colourdiv.setAttribute("class", "colour_block");
            colourdiv.setAttribute("style", `background-color: ${colour};`);

            div.appendChild(colourdiv);
        }

        // Create description paragraph
        let descpara = document.createElement("p");

        let pretty = pretty_desc(desc, type);

        descpara.setAttribute("class", "desc");
        descpara.innerText = pretty;

        div.appendChild(descpara);

        // Draw icon
        let icon = type_to_icon(type);

        if (icon) {
            let iconspan = document.createElement("p");

            iconspan.setAttribute("class", "icon");
            iconspan.innerText = icon;

            div.appendChild(iconspan);
        }

        // Create percentage span
        let pctspan = document.createElement("p");

        pctspan.setAttribute("id", `pct${index}`);

        div.appendChild(pctspan);

        // Add the divs
        reldiv.appendChild(div);
        elem.appendChild(reldiv);
    }

    // Set up pause/play button
    setup_pause();

    let pause = document.getElementById("pause");
    pause.onclick = pause_click;
    pause.style.display = "block";
}

function set_to_colour(set) {
    switch (set) {
        case 'A':
            return "rgb(140,87,60)";
        case 'B':
            return "rgb(181,223,249)";
        case 'C':
            return "rgb(200,71,147)";
        case 'D':
            return "rgb(233,153,63)";
        case 'E':
            return "rgb(218,56,51)";
        case 'F':
            return "rgb(252,243,80)";
        case 'G':
            return "rgb(85,176,99)";
        case 'H':
            return "rgb(48,112,182)";
    }
}

function type_to_icon(type) {
    switch (type) {
        case 'U':
            return "💡";
        case 'u':
            return "🛁";
        case 'R':
            return "🚂";
        case 'c':
            return "?";
        case 'C':
            return "🏆";
        case 'T':
            return "💠";
        case 't':
            return "💍";
        case 'J':
            return "▥";
        case "G":
            return "←";//👈";
        case 'g':
            return "👮‍♂️";
        case 'F':
            return "🚗";
    };
}

function pretty_desc(desc, type) {
    switch (type) {
        case 'P':
            switch (desc) {
                case "A1":
                    return "Old Kent Road";
                case "A2":
                    return "Whitechapel Road";
                case "B1":
                    return "The Angel Islington";
                case "B2":
                    return "Euston Road";
                case "B3":
                    return "Pentonville Road";
                case "C1":
                    return "Pall Mall";
                case "C2":
                    return "Whitehall";
                case "C3":
                    return "Nothumberland Avenue";
                case "D1":
                    return "Bow Street";
                case "D2":
                    return "Marlborough Street";
                case "D3":
                    return "Vine Street";
                case "E1":
                    return "Strand";
                case "E2":
                    return "Fleet Street";
                case "E3":
                    return "Trafalgar Square";
                case "F1":
                    return "Leicster Square";
                case "F2":
                    return "Coventry Street";
                case "F3":
                    return "Picadilly";
                case "G1":
                    return "Regent Street";
                case "G2":
                    return "Oxford Street";
                case "G3":
                    return "Bond Street";
                case "H1":
                    return "Park Lane";
                case "H2":
                    return "Mayfair";
            }
        case 'R':
            switch (desc) {
                case "R1":
                    return "Kings Cross Station";
                case "R2":
                    return "Marylebone Station";
                case "R3":
                    return "Fenchurch St. Station";
                case "R4":
                    return "Liverpool St. Station";
            }
    }

    return desc
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

        let addelem;

        if (square_type[elem] == 'P') {
            let colour = set_to_colour(square_desc[elem][0]);
            addelem = document.createElement("span");
            addelem.setAttribute("class", "colsample");
            addelem.setAttribute("style", `background-color: ${colour}`);
        }

        add_leaderboard(tbody, pretty_desc(square_desc[elem], square_type[elem]), stat, stats.moves, false, addelem)

        let reasons = stats.reasons[elem];

        if (square_type[elem] == 'J') {
            let visits = stat - reasons.reduce((a, b) => a + b, 0n);
            add_leaderboard(tbody, "Just Visiting", visits, stat, true)
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

            add_leaderboard(tbody, desc, count, stat, true)
        }
    }

    if ((stats.turns % 100_000_000n) == 0) {
        // Auto-pause at 100,000,000
        pause_click();
    }
}

function add_leaderboard(tbody, desc, value, total, sub, addelem) {
    let tr = document.createElement("tr");

    if (sub) {
        tr.setAttribute("class", "substat");
    }

    tbody.appendChild(tr);

    let td = document.createElement("td");
    td.setAttribute("class", "statlabel");

    if (addelem) {
        td.appendChild(addelem);
        let span = document.createElement("span");
        span.innerText = `${desc}:`;
        td.appendChild(span);
    } else {
        td.innerText = `${desc}:`;
    }

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