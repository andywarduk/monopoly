const debug = false; // Set to true to debug this script
const workerdebug = false; // Set to true to debug worker

const iterations = 10_000; // Number of iterations to perform in each chunk

// Spinner state
let spinner = true;

// Button toggles
let paused = false;
let jailwait = false;
let split_just_visiting = true;
let full_leaderboard = false;

// Worker thread object
let worker;

// Description of each space
let square_desc;

// Type of each space
let square_type;

// Index of needed squares
let square_visit;
let square_g2j;

// Arrival reason element descriptions
let arrival_reason_descs;

// Last statistics gathered
let last_stats;

// Expected probabilities for arriving at each space
let expected_freq;

// Number formatters
let number_formatter;
let percent_formatters = {};

// Run setup
setup();

function setup() {
    // Set up number formatters
    number_formatter = Intl.NumberFormat();

    // Set up web worker
    if (window.Worker) {
        worker = new Worker("$link(worker.js)", { type: "module" });
        worker.onmessage = process_worker_message;
    } else {
        alert("Web workers not available")
    }
}

function process_worker_message(msg) {
    if (debug) {
        console.debug("Got message from worker:", msg);
    }

    switch (msg.data.msgtype) {
        case "loaded":
            // Worker is loaded
            if (debug) {
                console.debug("Got 'loaded' from worker:", msg.data);
            }

            // Initialise the worker
            worker_init(true);

            break;

        case "initfin":
        case "reinitfin":
            // Process initialisation result
            if (debug) {
                console.debug(`Got ${msg.data.msgtype} from worker:`, msg.data);
            }

            if (msg.data.msgtype == "initfin") {
                // Set up the board after first initialise
                setup_board(msg.data);
            }

            // Get worker to calculate expected frequencies
            spinner_message("Calculating expected frequencies...");

            // Ask worker to calculate expected frequencies
            worker.postMessage({ msgtype: "calcexpected", jailwait: jailwait });

            break;

        case "calcexpectedfin":
            // Process expected frequencies result
            if (debug) {
                console.debug(`Got 'calcexpectedfin' from worker:`, msg.data);
            }

            // Save expected frequencies
            expected_freq = msg.data.freq;

            // Hide spinner
            spinner_show(false);

            // Start worker executing
            if (!paused) {
                worker.postMessage({ msgtype: "exec", ticks: iterations });
            }

            break;

        case "execfin":
            // Execute chunk finished
            if (debug) {
                console.debug("Got 'execfin' from worker:", msg.data);
            }

            if (!paused && !spinner) {
                // Execute next chunk
                worker.postMessage({ msgtype: "exec", ticks: iterations });
            }

            // Process the stats
            last_stats = msg.data.stats;
            process_stats(last_stats);

            break;

        default:
            // Unrecognised message
            console.error("Invalid message from worker:", msg);
            break;

    }
}

function worker_init(first) {
    // Update spinner
    spinner_message("Initialising...");
    spinner_show(true);

    // Tell worker to (re)initialise
    worker.postMessage({ msgtype: (first ? "init" : "reinit"), jailwait: jailwait, debug: workerdebug })
}

// Set up board squares
function setup_board(data) {
    // Save space data
    square_desc = data.square_desc;
    square_type = data.square_type;

    square_visit = square_type.findIndex((s) => s == 'J');
    square_g2j = square_type.findIndex((s) => s == 'g');

    // Save arrival reason descriptions
    arrival_reason_descs = data.arrival_reason_descs;

    for (const [index, desc] of square_desc.entries()) {
        // Get space type
        const type = square_type[index];

        // Find space table cell
        const elem = document.getElementById(index.toString());

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
        const reldiv = document.createElement("div");
        reldiv.setAttribute("class", `space_reldiv space_reldiv_${orient} space_reldiv_${side}`);

        const div = document.createElement("div");
        div.setAttribute("class", `space_div space_div_${orient} space_div_${side}`);

        // Add colour block for property squares
        if (type == 'P') {
            // Work out colour
            const colour = set_to_colour(desc[0]);

            // Add the div
            const colourdiv = document.createElement("div");

            colourdiv.setAttribute("class", "colour_block");
            colourdiv.setAttribute("style", `background-color: ${colour};`);

            div.appendChild(colourdiv);
        }

        // Create description paragraph
        const descpara = document.createElement("p");

        const pretty = pretty_desc(desc, type);

        descpara.setAttribute("class", "propname");
        descpara.innerHTML = pretty;

        div.appendChild(descpara);

        // Draw icon
        let icon = type_to_icon(type);

        if (icon) {
            const iconspan = document.createElement("p");

            iconspan.setAttribute("class", "icon");
            iconspan.innerHTML = icon;

            div.appendChild(iconspan);
        }

        // Create percentage span
        create_pct_span(div, index);

        if (type == 'J') {
            for (let i = 1; i <= 2; i++) {
                create_pct_span(div, index, i);
            }
        }

        // Add the divs
        reldiv.appendChild(div);
        elem.appendChild(reldiv);
    }

    // Set up pause/play button
    update_pause_button();

    const pause = document.getElementById("pause");
    pause.onclick = pause_click;

    // Set up split jail stats button
    update_jailstats_button();

    const jailstats = document.getElementById("splitjail");
    jailstats.onclick = jailstats_click;

    // Set up full leaderboard button
    update_fullboard_button();

    const fullboard = document.getElementById("fullboard");
    fullboard.onclick = fullboard_click;

    // Set up strategy button
    update_strategy_button();

    const strategy = document.getElementById("strategy");
    strategy.onclick = strategy_click;

    // Display the board
    const main = document.getElementById("main");
    main.style.display = "flex";
}

function create_pct_span(parent, index, sub) {
    const pctdiv = document.createElement("div");
    const pctspan = document.createElement("span");

    let id;

    if (sub) {
        id = `pct${index}-${sub}`;
    } else {
        id = `pct${index}`;
    }

    pctspan.setAttribute("id", id);
    pctspan.setAttribute("class", "pct_span");

    pctdiv.appendChild(pctspan);
    parent.appendChild(pctdiv);
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

function pretty_desc(desc, type, show_elem) {
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
        case 'C':
            if (show_elem) {
                return `Community Chest ${desc.substring(1)}`;
            } else {
                return "Community Chest";
            }
        case 'c':
            if (show_elem) {
                return `Chance ${desc.substring(1)}`;
            } else {
                return "Chance";
            }
    }

    return desc
}

function process_stats(stats) {
    // Make sure these stats are for the mode we're currently in
    if (stats.jailwait != jailwait) {
        console.warn("Rejecting stats for incorrect mode");
        return;
    }

    if (debug) {
        console.debug("Processing stats:", stats);
    }

    // Update game statistics
    //                2 rolls            3 rolls                   3 rolls (goes to jail on 3rd roll)
    const doubles_tot = stats.doubles[0] + (2n * stats.doubles[1]) + (3n * stats.doubles[2]);

    const double_turns = stats.doubles[0];
    const triple_turns = stats.doubles[1] + stats.doubles[2];
    const single_turns = stats.turns - (double_turns + triple_turns);

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

    // Rank the spaces by arrivals
    for (const [index, arrivals] of stats.arrivals.entries()) {
        switch (square_type[index]) {
            case 'J': // Just visiting
                if (split_just_visiting) {
                    leaderboard.push([index, arrivals, 2]);
                }

                break;
            case 'g': // Go to Jail
                leaderboard.push([index, 0n, 0]);

                if (split_just_visiting) {
                    // Jail (visit sub 2)
                    leaderboard.push([square_visit, arrivals, 1]);
                } else {
                    // Combined jail + just visiting
                    leaderboard.push([square_visit, arrivals + stats.arrivals[square_visit], 0]);
                }

                break;
            default:
                leaderboard.push([index, arrivals, 0]);
        }
    };

    // Sort by arrivals
    leaderboard.sort(([_ia, aa, _sa], [_ib, ab, _sb]) => Number(ab - aa));

    // Draw colour ranked percentages on board squares
    const split = 180 / leaderboard.length;

    for (const [rank, [index, arrivals, sub]] of leaderboard.entries()) {
        let id;

        if (sub == 0) {
            id = `pct${index}`;
        } else {
            id = `pct${index}-${sub}`;
        }

        const elem = document.getElementById(id);
        const colour = `hsl(${rank * split}, 100%, 60%)`;

        elem.style.backgroundColor = colour;
        elem.innerText = percent(arrivals, stats.moves);
    };

    // Clear the leaderboard
    const tbody = document.getElementById("leaderboard");
    tbody.textContent = "";

    // Get top 20 or full
    for (let i = 0; i < (full_leaderboard ? leaderboard.length : 20); i++) {
        const [elem, stat, sub] = leaderboard[i];

        // Create colour swatch for properties
        let addelem;

        if (square_type[elem] == 'P') {
            const colour = set_to_colour(square_desc[elem][0]);
            addelem = document.createElement("span");
            addelem.setAttribute("class", "colsample");
            addelem.setAttribute("style", `background-color: ${colour}`);
        }

        // Get space description
        let desc;

        if (sub == 2) {
            desc = "Just Visiting";
        } else {
            desc = pretty_desc(square_desc[elem], square_type[elem], true);
        }

        // Get expected frequency
        let expected;

        if (square_type[elem] == 'g') {
            // Go to jail
            expected = 0;
        } else if (square_type[elem] == 'J') {
            // Jail
            switch (sub) {
                case 0: // Combined jail/just visiting
                    expected = expected_freq[square_visit] + expected_freq[square_g2j];
                    break;
                case 1: // In jail
                    expected = expected_freq[square_g2j];
                    break;
                case 2: // Just visiting
                    expected = expected_freq[square_visit];
                    break;
            }
        } else {
            expected = expected_freq[elem];
        }

        // Add leaderboard main entry
        add_leaderboard(tbody, desc, stat, stats.moves, false, expected, addelem)

        // Get arrival reasons
        let reasons;

        if (square_type[elem] == 'J' && sub !== 2) {
            // Get reasons from go to jail for jail
            reasons = stats.reasons[square_g2j];
        } else if (square_type[elem] == 'g') {
            // No reasons for actual go to jail
            reasons = [];
        } else {
            reasons = stats.reasons[elem];
        }

        // Special handling for Just Visiting for Jail space
        if (!split_just_visiting && square_type[elem] == 'J') {
            add_leaderboard(tbody, "Just Visiting", stats.arrivals[square_visit], stat, true);
        }

        // Add arrival reasons
        for (const [index, count] of reasons.entries()) {
            if (count == 0) {
                // Skip zeroes
                continue;
            }

            add_leaderboard(tbody, arrival_reason_descs[index], count, stat, true)
        }
    }

    // Roll frequencies
    const rolls = stats.rollfreq;

    // Calculate maximum number of rolls
    const max_rolsl = rolls.reduce((max, r) => {
        if (r > max) {
            return Number(r);
        } else {
            return max;
        }
    }, 0);

    for (const [i, count] of rolls.entries()) {
        const dice_sum = i + 2;

        // Calculate percentages
        const pct = percent_calc(count, stats.moves);
        const barpct = percent_calc(count, max_rolsl) * 100;

        // Size bar chart bar
        const graphbar = document.getElementById(`rollgraph${dice_sum}`);
        graphbar.style.height = `${barpct}%`;

        // Sort out bar borders
        if (i < 11 && count < rolls[i + 1]) {
            graphbar.style.borderRight = "0px";
        }

        if (i > 0 && count < rolls[i - 1]) {
            graphbar.style.borderLeft = "0px";
        }

        // Draw percentage
        const pctcell = document.getElementById(`rollpct${dice_sum}`);
        pctcell.innerText = percent_fmt(pct, 4);

        // Calculat expected
        let numerator;

        if (dice_sum <= 7) {
            numerator = dice_sum - 1;
        } else {
            numerator = 13 - dice_sum;
        }

        const expected = numerator / 36;

        // Calculate error
        const error = pct - expected;

        // Draw error
        const err = document.getElementById(`rollpcterr${dice_sum}`);
        err.innerText = percent_fmt(error, 4);
        colour_error(err, error, 6);
    }

    // Auto-pause at 100,000,000
    if (((stats.turns + BigInt(iterations)) % 100_000_000n) == 0) {
        pause_click();
    }
}

function roundnum(num, dp) {
    let mult = Math.pow(10, dp);
    let result = Math.round((num + Number.EPSILON) * mult) / mult;

    if (result === 0) {
        // Turn negative zero in to positive zero
        return 0
    }

    return result;
}

function colour_error(elem, error, dp) {
    let rnderr = roundnum(error, dp);

    switch (Math.sign(rnderr)) {
        case -1:
            elem.style.color = "red";
            break;
        case 0:
            elem.style.color = "black";
            break;
        case 1:
            elem.style.color = "green";
            break;
    }
}

function update_stat(id, value, total) {
    const elem = document.getElementById(id);
    elem.innerText = number_formatter.format(value);

    if (total !== undefined) {
        const telem = document.getElementById(`${id}_pct`);
        telem.innerText = percent(value, total);
    }
}

function add_leaderboard(tbody, desc, value, total, sub, expected, addelem) {
    // Create table row
    const tr = document.createElement("tr");

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
    } else {
        // Add text
        td.innerHTML = desc;
    }

    tr.appendChild(td);

    // Create number cell
    td = document.createElement("td");

    td.setAttribute("class", "stat");
    td.innerText = number_formatter.format(value);

    tr.appendChild(td);

    // Create percentage cell
    td = document.createElement("td");

    td.setAttribute("class", "statpct");
    const pct = percent_calc(value, total);
    td.innerText = percent_fmt(pct, 3);

    tr.appendChild(td);

    if (expected !== undefined) {
        // Create expected cell
        td = document.createElement("td");

        td.setAttribute("class", "statpct");
        td.innerText = percent_fmt(expected, 3);

        tr.appendChild(td);

        // Create error cell
        td = document.createElement("td");

        td.setAttribute("class", "statpct");
        const error = pct - expected;
        td.innerText = percent_fmt(error, 4);
        colour_error(td, error, 6);

        tr.appendChild(td);
    }
}

// Percentage calculation and display

function percent(value, total, dp) {
    // Return locale specific percentage
    return percent_fmt(percent_calc(value, total), dp);
}

function percent_calc(value, total) {
    if (total == 0) {
        return 0;
    } else {
        return Number(value) / Number(total);
    }
}

function percent_fmt(value, dp) {
    dp = dp || 2;

    let formatter = percent_formatters[dp];

    if (!formatter) {
        formatter = Intl.NumberFormat(undefined, { style: "percent", "minimumFractionDigits": dp, "maximumFractionDigits": dp });
        percent_formatters[dp] = formatter;
    }

    return formatter.format(roundnum(value, dp + 2));
}

// Loading spinner

function spinner_message(msg) {
    const elem = document.getElementById("spinnermsg");
    elem.innerText = msg;
}

function spinner_show(visible) {
    if (visible != spinner) {
        const elem = document.getElementById("loading");

        if (visible) {
            elem.style.display = "";
            document.body.style.overflow = "hidden";
        } else {
            elem.style.display = "none";
            document.body.style.overflow = "";
        }

        spinner = visible;
    }
}

// Pause/Play button

function pause_click() {
    // Pause/play button click handler
    paused = !paused;

    update_pause_button();

    if (!paused) {
        worker.postMessage({ msgtype: "exec", ticks: iterations });
    }
}

function update_pause_button() {
    // Update pause/play button
    const pause = document.getElementById("pause");

    if (!paused) {
        pause.innerText = "⏸︎ Pause";
    } else {
        pause.innerText = "▶ Play";
    }
}

// Combined Jail/Just Visiting control

function jailstats_click() {
    // Jail stats button click handler
    split_just_visiting = !split_just_visiting;

    update_jailstats_button();
}

function update_jailstats_button() {
    // Update jail stats button
    const btn = document.getElementById("splitjail");

    const index = square_type.findIndex((e) => e == 'J');

    if (split_just_visiting) {
        btn.innerText = "Combine Just Visiting";
    } else {
        btn.innerText = "Split Just Visiting";
    }

    let elem;

    elem = document.getElementById(`pct${index}`);
    elem.style.display = (split_just_visiting ? "none" : "inline");
    elem = document.getElementById(`pct${index}-1`);
    elem.style.display = (split_just_visiting ? "inline" : "none");
    elem = document.getElementById(`pct${index}-2`);
    elem.style.display = (split_just_visiting ? "inline" : "none");

    if (last_stats) {
        process_stats(last_stats);
    }
}

// Full leaderboard control

function fullboard_click() {
    // Full leaderboard button click handler
    full_leaderboard = !full_leaderboard;

    update_fullboard_button();
}

function update_fullboard_button() {
    // Update full leaderboard button
    const btn = document.getElementById("fullboard");

    if (full_leaderboard) {
        btn.innerText = "Top 15 Only";
    } else {
        btn.innerText = "Full Leaderboard";
    }

    if (last_stats) {
        process_stats(last_stats);
    }
}

// Strategy control

function strategy_click() {
    // Strategy button click handler
    jailwait = !jailwait;

    update_strategy_button();

    if (paused) {
        // Unpause
        paused = false;
        update_pause_button();
    }

    // Re-initialise
    worker_init(false);
}

function update_strategy_button() {
    // Update strategy button
    const btn = document.getElementById("strategy");

    if (jailwait) {
        btn.innerText = "Pay to Exit Jail";
    } else {
        btn.innerText = "Roll to Exit Jail";
    }
}
