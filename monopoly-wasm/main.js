let debug = false;
let paused = false;
let split_just_visiting = true;
let full_leaderboard = false;
const iterations = 1000;
let worker;
let square_desc;
let square_type;
let last_stats;

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

            if (!paused) {
                // Execute next chunk
                worker.postMessage({ msgtype: "execute", ticks: iterations });
            }

            // Process the stats
            last_stats = msg.data.stats;
            process_stats(last_stats);

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

        descpara.setAttribute("class", "propname");
        descpara.innerHTML = pretty;

        div.appendChild(descpara);

        // Draw icon
        let icon = type_to_icon(type);

        if (icon) {
            let iconspan = document.createElement("p");

            iconspan.setAttribute("class", "icon");
            iconspan.innerHTML = icon;

            div.appendChild(iconspan);
        }

        // Create percentage span
        let pctspan = document.createElement("p");

        pctspan.setAttribute("id", `pct${index}`);

        div.appendChild(pctspan);

        if (type == 'J') {
            for (let i = 1; i <= 2; i++) {
                let pctspan = document.createElement("p");

                pctspan.setAttribute("id", `pct${index}-${i}`);

                div.appendChild(pctspan);
            }
        }

        // Add the divs
        reldiv.appendChild(div);
        elem.appendChild(reldiv);
    }

    // Set up pause/play button
    update_pause_button();

    let pause = document.getElementById("pause");
    pause.onclick = pause_click;
    pause.style.display = "block";

    // Set up split jail stats button
    update_jailstats_button();

    let jailstats = document.getElementById("splitjail");
    jailstats.onclick = jailstats_click;
    jailstats.style.display = "block";

    // Set up full leaderboard button
    update_fullboard_button();

    let fullboard = document.getElementById("fullboard");
    fullboard.onclick = fullboard_click;
    fullboard.style.display = "block";
}

function set_to_colour(set) {
    // Convert property set letter to colour
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
    // Convert space type to icon
    switch (type) {
        case 'U':
            return "💡";
        case 'u':
            return "🛁";
        case 'R':
            return "🚂";
        case 'c':
            return "<span style='font-size: 50px'>?</span>";
        case 'C':
            return "🏆";
        case 'T':
            return "💠";
        case 't':
            return "💍";
        case 'J':
            return "<span style='font-size: 50px'>▥</span>";
        case "G":
            return "<span style='font-size: 50px'>←</span>"
        case 'g':
            return "👮‍♂️";
        case 'F':
            return "🚗";
    };
}

function pretty_desc(desc, type) {
    // Return description of property according to UK Monopoly version
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
                    return "Nothumber&shy;land Avenue";
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

    // Update game statistics
    //                2 rolls            3 rolls                   3 rolls (goes to jail on 3rd roll)
    let doubles_tot = stats.doubles[0] + (2n * stats.doubles[1]) + (3n * stats.doubles[2]);

    let double_turns = stats.doubles[0]
    let triple_turns = stats.doubles[1] + stats.doubles[2]
    let single_turns = stats.turns - (double_turns + triple_turns);

    update_stat("stat_turns", stats.turns);

    update_stat("stat_turns_single", single_turns, stats.turns);
    update_stat("stat_turns_double", double_turns, stats.turns);
    update_stat("stat_turns_triple", triple_turns, stats.turns);
    update_stat("stat_ddoubles", stats.doubles[1], stats.turns);
    update_stat("stat_tdoubles", stats.doubles[2], stats.turns);

    update_stat("stat_moves", stats.moves);

    update_stat("stat_doubles_tot", doubles_tot, stats.moves);

    // Calculate leaderboard
    let leaderboard = [];

    for (const [index, arrivals] of stats.arrivals.entries()) {
        if (split_just_visiting && square_type[index] == 'J') {
            let reasons = stats.reasons[index];

            let jail = reasons.reduce((a, b) => a + b, 0n);
            let visits = arrivals - jail;

            leaderboard.push([index, visits, 2]);
            leaderboard.push([index, jail, 1]);
        } else {
            leaderboard.push([index, arrivals, 0]);
        }
    };

    leaderboard.sort(([_ia, aa, _sa], [_ib, ab, _sb]) => Number(ab - aa));

    // Draw percentages on board squares
    let split = 180 / leaderboard.length;

    for (const [rank, [index, arrivals, sub]] of leaderboard.entries()) {
        let id;

        if (sub == 0) {
            id = `pct${index}`;
        } else {
            id = `pct${index}-${sub}`;
        }

        let elem = document.getElementById(id);
        let colour = `hsl(${rank * split}, 100%, 60%)`;

        elem.innerHTML = `<span class="pct_span" style="background-color: ${colour}">${percent(arrivals, stats.moves)}</span>`;
    };

    // Clear the leaderboard
    let container = document.getElementById("leaderboard");
    container.innerHTML = "";

    // Create new leaderboard table
    let table = document.createElement("table");
    container.appendChild(table);

    let tbody = document.createElement("tbody");
    table.appendChild(tbody);

    // Get top 15 or full
    for (let i = 0; i < (full_leaderboard ? leaderboard.length : 15); i++) {
        let elem = leaderboard[i][0];
        let stat = leaderboard[i][1];
        let sub = leaderboard[i][2];

        let addelem;

        // Create colour swatch for properties
        if (square_type[elem] == 'P') {
            let colour = set_to_colour(square_desc[elem][0]);
            addelem = document.createElement("span");
            addelem.setAttribute("class", "colsample");
            addelem.setAttribute("style", `background-color: ${colour}`);
        }

        let desc;

        if (sub == 2) {
            desc = "Just Visiting";
        } else {
            desc = pretty_desc(square_desc[elem], square_type[elem]);
        }

        // Add leaderboard main entry
        add_leaderboard(tbody, desc, stat, stats.moves, false, addelem)

        if (sub == 2) {
            // Skip reasons for just visiting
            continue;
        }

        // Get arrival reasons
        let reasons = stats.reasons[elem];

        // Special handling for Just Visiting for Jail space
        if (!split_just_visiting && square_type[elem] == 'J') {
            let visits = stat - reasons.reduce((a, b) => a + b, 0n);
            add_leaderboard(tbody, "Just Visiting", visits, stat, true)
        }

        // Add arrival reasons
        for (const [index, count] of reasons.entries()) {
            if (count == 0) {
                // Skip zeroes
                continue;
            }

            add_leaderboard(tbody, arrival_reason_desc(index), count, stat, true)
        }
    }

    if (((stats.turns + BigInt(iterations)) % 100_000_000n) == 0) {
        // Auto-pause at 100,000,000
        pause_click();
    }
}

function update_stat(id, value, total) {
    let elem = document.getElementById(id);
    elem.innerText = value.toLocaleString();

    if (total) {
        elem = document.getElementById(`${id}_pct`);
        elem.innerText = percent(value, total);
    }
}

function add_leaderboard(tbody, desc, value, total, sub, addelem) {
    // Create table row
    let tr = document.createElement("tr");

    if (sub) {
        // Sub stat - add class
        tr.setAttribute("class", "substat");
    }

    tbody.appendChild(tr);

    // Create description cell
    let td = document.createElement("td");
    td.setAttribute("class", "statlabel");

    if (addelem) {
        // Add text
        let span = document.createElement("span");
        span.innerHTML = `${desc}`;
        td.appendChild(span);

        // Add additional element
        td.appendChild(addelem);

        // Add colon
        span = document.createElement("span");
        span.innerHTML = ":";
        td.appendChild(span);
    } else {
        // Add text with colon
        td.innerHTML = `${desc}:`;
    }

    tr.appendChild(td);

    // Create number cell
    td = document.createElement("td");

    td.setAttribute("class", "stat");
    td.innerText = value.toLocaleString();

    tr.appendChild(td);

    // Create percentage cell
    td = document.createElement("td");

    td.setAttribute("class", "statpct");
    td.innerText = percent(value, total, 3);

    tr.appendChild(td);
}

function arrival_reason_desc(index) {
    // Convert arrival reason to string
    switch (index) {
        case 0:
            return "Chance Card";
        case 1:
            return "Community Chest Card";
        case 2:
            return "Go to Jail";
        case 3:
            return "Triple Double";
    }

    return "<Unknown>";
}

function percent(value, total, dp) {
    // Return locale specific percentage
    let percentage;

    if (total == 0) {
        percentage = 0;
    } else {
        percentage = Number(value) / Number(total);
    }

    dp = dp || 2;

    return percentage.toLocaleString(undefined, { style: "percent", "minimumFractionDigits": dp, "maximumFractionDigits": dp });
}

function pause_click() {
    // Pause/play button click handler
    paused = !paused;

    update_pause_button();

    if (!paused) {
        worker.postMessage({ msgtype: "execute", ticks: iterations });
    }
}

function update_pause_button() {
    // Update pause/play button
    let pause = document.getElementById("pause");

    if (!paused) {
        pause.innerText = "⏸︎ Pause";
    } else {
        pause.innerText = "▶ Play";
    }
}

function jailstats_click() {
    // Jail stats button click handler
    split_just_visiting = !split_just_visiting;

    update_jailstats_button();
}

function update_jailstats_button() {
    // Update jail stats button
    let btn = document.getElementById("splitjail");

    let index = square_type.findIndex((e) => e == 'J');

    if (split_just_visiting) {
        btn.innerText = "Combine Just Visiting";
    } else {
        btn.innerText = "Split Just Visiting";
    }

    let elem;

    elem = document.getElementById(`pct${index}`);
    elem.style.display = (split_just_visiting ? "none" : "block");
    elem = document.getElementById(`pct${index}-1`);
    elem.style.display = (split_just_visiting ? "block" : "none");
    elem = document.getElementById(`pct${index}-2`);
    elem.style.display = (split_just_visiting ? "block" : "none");

    if (last_stats) {
        process_stats(last_stats);
    }
}

function fullboard_click() {
    // Full leaderboard button click handler
    full_leaderboard = !full_leaderboard;

    update_fullboard_button();
}

function update_fullboard_button() {
    // Update full leaderboard button
    let btn = document.getElementById("fullboard");

    if (full_leaderboard) {
        btn.innerText = "Top 15 Only";
    } else {
        btn.innerText = "Full Leaderboard";
    }

    if (last_stats) {
        process_stats(last_stats);
    }
}
