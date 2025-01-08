import { perf_now, perf_end, perf_mark } from "$link(perf.js)";

const debug = false; // Set to true to debug this script
const workerdebug = false; // Set to true to debug worker

const turn_chunk = 100_000; // Number of iterations to perform in each chunk
const autopause = 100_000_000; // Number of iterations before automatically pausing
let turn_target; // Next iteration target

// Spinner state
let spinner = true;

// Button toggles
let paused = false;
let jailwait = false;
let split_just_visiting = true;
let full_leaderboard = false;

// Worker thread object
let worker;

// Code of each space
let space_codes;

// Indexes of needed spaces
let space_visit;
let space_g2j;

// Arrival reason element descriptions
let arrival_reason_descs;

// Last statistics gathered
let last_stats;

// Expected probabilities for arriving at each space
let expected_freq;

// Number formatters
const number_formatter = Intl.NumberFormat();
let percent_formatters = {};

// Main setup function
(() => {
    // Set up web worker
    if (window.Worker) {
        worker = new Worker("$link(worker.js)", { type: "module" });

        worker.onerror = (e) => {
            console.error("Worker raised an error:", e);
            alert("Worker raised an error: " + e.message);
        }

        worker.onmessageerror = (e) => {
            console.error("Worker raised a message error:", e);
            alert("Worker raised a message error: " + e.message);
        }

        worker.onmessage = process_worker_message;
    } else {
        alert("Web workers not available")
    }
})();

// Process message received from the worker thread
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

            // Start worker executing to first autopause target
            turn_target = autopause;

            if (!paused) {
                worker_exec_start();
            }

            break;

        case "execfin":
            // Execute chunk finished
            if (debug) {
                console.debug("Got 'execfin' from worker:", msg.data);
            }

            // Make sure these stats are for the strategy mode we're currently in
            if (msg.data.jailwait != jailwait) {
                console.warn("Rejecting stats for incorrect strategy");
                break;
            }

            // Save stats
            last_stats = msg.data.stats;

            // Autopaused?
            if (msg.data.paused) {
                paused = true;
                turn_target += autopause;
                update_pause_button();
            }

            // Extract number of turns from stats
            let turns = msg.data.stats.turns;

            // Save time before requesting animation
            const start_time = perf_now();

            // Request animation frame to update the page
            requestAnimationFrame((ts) => {
                // Record time to get animation frame
                perf_end(`requestAnimation${turns}`, start_time);

                animationFrame(ts, turns)
            });

            break;

        default:
            // Unrecognised message
            console.error("Invalid message from worker:", msg);
            break;

    }
}

// Initialise or re-initialise the worker thread
function worker_init(first) {
    // Update spinner
    spinner_message("Initialising...");
    spinner_show(true);

    // Tell worker to (re)initialise
    worker.postMessage({ msgtype: (first ? "init" : "reinit"), jailwait: jailwait, debug: workerdebug })
}

// Get the worker to execute a chunk
function worker_exec_start() {
    perf_mark("postMessageExecStart");
    worker.postMessage({ msgtype: "execstart", target_turns: turn_target, chunk_size: turn_chunk });
}

function worker_exec_stop() {
    perf_mark("postMessageExecStop");
    worker.postMessage({ msgtype: "execstop" });
}

// Set up board spaces with data returned from worker initialisation
function setup_board(data) {
    // Save space data
    space_codes = data.spaces;

    space_visit = space_codes.findIndex((s) => s == "J");
    space_g2j = space_codes.findIndex((s) => s == "g");

    // Save arrival reason descriptions
    arrival_reason_descs = data.arrival_reasons;

    // Create the board
    let table = document.getElementById("board_tab");

    let tbody = document.createElement("tbody");

    let i, tr;

    // Top row
    tr = document.createElement("tr");

    for (i = 0; i <= 10; i++) {
        tr.appendChild(create_space_td(i));
    }

    tbody.appendChild(tr);

    // Sides
    for (i = 0; i <= 8; i++) {
        tr = document.createElement("tr");

        tr.appendChild(create_space_td(39 - i));

        if (i == 0) {
            tr.appendChild(create_title_td())
        }

        tr.appendChild(create_space_td(11 + i));

        tbody.appendChild(tr);
    }

    table.appendChild(tbody);

    // Bottom row
    tr = document.createElement("tr");

    for (i = 0; i <= 10; i++) {
        tr.appendChild(create_space_td(30 - i));
    }

    tbody.appendChild(tr);

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

// Creates the title cell
function create_title_td() {
    const td = document.createElement("td");

    td.colSpan = 9;
    td.rowSpan = 9;

    const div1 = document.createElement("div");
    div1.setAttribute("class", "title_div");

    const div2 = document.createElement("div");
    div1.setAttribute("class", "title_border");

    const h1 = document.createElement("h1");
    h1.setAttribute("class", "title");
    h1.innerText = "MONOPOLY";

    div2.appendChild(h1);

    div1.appendChild(div2);

    td.appendChild(div1);

    return td;
}

// Create board space table cell
function create_space_td(index) {
    const td = document.createElement("td");
    td.setAttribute("class", "space_cell greenbg");
    td.appendChild(create_space(index));

    return td;
}

// Create board space content for a given board space index
function create_space(index) {
    // Get space code
    const code = space_codes[index];

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

    // Add colour block for property spaces
    if (code[0] == 'P') {
        // Work out colour
        const colour = set_to_colour(code[1]);

        // Add the div
        const colourdiv = document.createElement("div");

        colourdiv.setAttribute("class", "colour_block");
        colourdiv.setAttribute("style", `background-color: ${colour};`);

        div.appendChild(colourdiv);
    }

    // Create description paragraph
    const descpara = document.createElement("p");

    descpara.setAttribute("class", "propname");
    descpara.innerHTML = space_code_to_description(code);

    div.appendChild(descpara);

    // Draw icon
    let icon = space_code_to_icon(code);

    if (icon) {
        const iconspan = document.createElement("p");

        iconspan.setAttribute("class", "icon");
        iconspan.innerHTML = icon;

        div.appendChild(iconspan);
    }

    // Create percentage span
    create_pct_span(div, index);

    if (code == 'J') {
        // Create subdivisions for Jail space
        for (let i = 1; i <= 2; i++) {
            create_pct_span(div, index, i);
        }
    }

    // Add the divs
    reldiv.appendChild(div);

    return reldiv;
}

// Creates a percentage span for a board space
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

// Convert property set letter to colour
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

// Convert board space code to icon
function space_code_to_icon(code) {
    switch (code[0]) {
        case 'U':
            switch (code) {
                case 'U1':
                    return "💡";
                case 'U2':
                    return "🛁";
            }
        case 'R':
            return "🚂";
        case 'c':
            return "<span style='font-size: 50px; color: black;'>?</span>";
        case 'C':
            return "🏆";
        case 'T':
            switch (code) {
                case 'T1':
                    return "💠";
                case 'T2':
                    return "💍";
            }
        case 'J':
            return "<span style='font-size: 50px; color: black;'>▥</span>";
        case "G":
            return "<span style='font-size: 50px; color: black;'>←</span>"
        case 'g':
            return "👮‍♂️";
        case 'F':
            return "🚗";
    };
}

// Return description of a space (London UK Monopoly version)
function space_code_to_description(code, show_elem) {
    switch (code[0]) {
        case 'G':
            return "Go";
        case 'J':
            return "Jail";
        case 'F':
            return "Free Parking";
        case 'g':
            return "Go to Jail";
        case 'P':
            switch (code.substring(1)) {
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
            switch (code) {
                case "R1":
                    return "Kings Cross Station";
                case "R2":
                    return "Marylebone Station";
                case "R3":
                    return "Fenchurch St. Station";
                case "R4":
                    return "Liverpool St. Station";
            }
        case 'U':
            switch (code) {
                case "U1":
                    return "Electric Company";
                case "U2":
                    return "Water Works";
            }
        case 'C':
            if (show_elem) {
                return `Community Chest ${code.substring(1)}`;
            } else {
                return "Community Chest";
            }
        case 'c':
            if (show_elem) {
                return `Chance ${code.substring(1)}`;
            } else {
                return "Chance";
            }
        case 'T':
            switch (code) {
                case "T1":
                    return "Income Tax";
                case "T2":
                    return "Luxury Tax";
            }
    }

    return "<Unknown>";
}

// requestAnimationFrame callback
function animationFrame(ts, turns) {
    // Is this the animation callback for the current stats?
    if (turns != last_stats.turns) {
        return
    }

    // Save time before updating the page
    const start_time = perf_now();

    // Process the statistics
    process_stats(last_stats);

    // Measure page update time
    perf_end(`pageUpdate${turns}`, start_time);
}

// Processes statistics returned by game tick
function process_stats(stats) {
    update_games_stats(stats);
    update_percentages_and_leaderboard(stats);
    update_roll_frequencies(stats);
}

// Update game staistics from passed stats
function update_games_stats(stats) {
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
}

// Update a game statistic with optional percentage
function update_stat(id, value, total) {
    const elem = document.getElementById(id);
    elem.innerText = number_formatter.format(value);

    if (total !== undefined) {
        const telem = document.getElementById(`${id}_pct`);
        telem.innerText = percent(value, total);
    }
}

// Update percentages on the board spaces and fill the leaderboard
function update_percentages_and_leaderboard(stats) {
    // Calculate leaderboard
    let leaderboard = [];

    // Rank the spaces by arrivals
    for (const [index, arrivals] of stats.arrivals.entries()) {
        switch (space_codes[index][0]) {
            case 'J': // Just visiting
                if (split_just_visiting) {
                    leaderboard.push([index, arrivals, 2]);
                }

                break;
            case 'g': // Go to Jail
                leaderboard.push([index, 0n, 0]);

                if (split_just_visiting) {
                    // Jail (visit sub 2)
                    leaderboard.push([space_visit, arrivals, 1]);
                } else {
                    // Combined jail + just visiting
                    leaderboard.push([space_visit, arrivals + stats.arrivals[space_visit], 0]);
                }

                break;
            default:
                leaderboard.push([index, arrivals, 0]);
        }
    };

    // Sort by arrivals
    leaderboard.sort(([_ia, aa, _sa], [_ib, ab, _sb]) => Number(ab - aa));

    // Draw colour ranked percentages on board spaces
    const hue_split = 180 / leaderboard.length;

    for (const [rank, [index, arrivals, sub]] of leaderboard.entries()) {
        let id;

        if (sub == 0) {
            id = `pct${index}`;
        } else {
            id = `pct${index}-${sub}`;
        }

        const elem = document.getElementById(id);
        const colour = `hsl(${rank * hue_split}, 100%, 60%)`;

        elem.style.backgroundColor = colour;
        elem.innerText = percent(arrivals, stats.moves);
    };

    // Clear the leaderboard table
    const tbody = document.getElementById("leaderboard");
    tbody.textContent = "";

    // Get top 20 or full
    for (let i = 0; i < (full_leaderboard ? leaderboard.length : 20); i++) {
        const [elem, stat, sub] = leaderboard[i];
        let code = space_codes[elem];

        // Get expected frequency
        let expected;

        switch (code) {
            case 'g': // Go to jail
                expected = 0;

                break;

            case 'J': // Jail
                switch (sub) {
                    case 0: // Combined jail/just visiting
                        expected = expected_freq[space_visit] + expected_freq[space_g2j];
                        break;
                    case 1: // In jail
                        expected = expected_freq[space_g2j];
                        break;
                    case 2: // Just visiting
                        expected = expected_freq[space_visit];
                        break;
                }

                break;

            default:
                expected = expected_freq[elem];

                break;

        }

        // Add leaderboard main entry
        add_leaderboard_row(tbody, 'S', [elem, sub], stat, stats.moves, expected)

        // Get arrival reasons
        let reasons;

        if (code == 'J' && sub !== 2) {
            // Get reasons from go to jail for jail
            reasons = stats.reasons[space_g2j];
        } else if (code == 'g') {
            // No reasons for actual go to jail
            reasons = [];
        } else {
            reasons = stats.reasons[elem];
        }

        let sort_reasons = [];

        // Special handling for Just Visiting for Jail space
        if (!split_just_visiting && code == 'J') {
            sort_reasons.push(['J', [], stats.arrivals[space_visit]]);
        }

        // Add arrival reasons
        for (const [index, count] of reasons.entries()) {
            if (count == 0) {
                // Skip zeroes
                continue;
            }

            sort_reasons.push(['R', [index, elem, sub], count]);
        }

        // Sort descending
        sort_reasons.sort((a, b) => Number(b[2]) - Number(a[2]));

        // Add to the leaderboard
        for (const [type, idxelems, count] of sort_reasons) {
            add_leaderboard_row(tbody, type, idxelems, count, stat);
        }
    }
}

// Update dice roll frequency statistics
function update_roll_frequencies(stats) {
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
}

// Cached leaderboard table rows
let leaderboard_row_cache = {};

// Adds a leaderboard row - either from the cache or create
function add_leaderboard_row(tbody, type, idxelems, value, total, expected) {
    // Look up leaderboard row in the cache
    let key = `${type}-${idxelems.join("-")}`;

    let row = leaderboard_row_cache[key];

    if (row === undefined) {
        // Not found - create it
        row = create_leaderboard_row(type, idxelems);
        leaderboard_row_cache[key] = row;
    }

    // Set value
    row.value.innerText = number_formatter.format(value);

    // Set percentage
    const pct = percent_calc(value, total);
    row.pct.innerText = percent_fmt(pct, 3);

    if (expected !== undefined) {
        // Set expected
        row.expected.innerText = percent_fmt(expected, 3);

        // Set error
        const error = pct - expected;
        row.error.innerText = percent_fmt(error, 4);
        colour_error(row.error, error, 6);
    } else {
        row.pct.innerText = "";
        row.error.innerText = "";
    }

    // Add row to table
    tbody.appendChild(row.tr);
}

// Creates a leaderboard table row for caching
function create_leaderboard_row(type, idxelems) {
    // Create table row
    const tr = document.createElement("tr");

    let minor = false;

    // Add description cell
    let desc_cell;

    switch (type) {
        case 'S':
            desc_cell = create_leaderboard_space_cell(idxelems[0], idxelems[1]);
            break;
        case 'J':
            desc_cell = create_leaderboard_text_cell("Just Visiting");
            minor = true;
            break;
        case 'R':
            desc_cell = create_leaderboard_text_cell(arrival_reason_descs[idxelems[0]]);
            minor = true;
            break;
    }

    if (minor) {
        // Sub stat - add class
        tr.setAttribute("class", "substat");
    }

    tr.appendChild(desc_cell);

    // Create number cell
    const value = document.createElement("td");
    value.setAttribute("class", "stat");
    tr.appendChild(value);

    // Create percentage cell
    const pct = document.createElement("td");
    pct.setAttribute("class", "statpct");
    tr.appendChild(pct);

    // Create expected cell
    const expected = document.createElement("td");
    expected.setAttribute("class", "statpct");
    tr.appendChild(expected);

    // Create error cell
    let error = document.createElement("td");
    error.setAttribute("class", "statpct");
    tr.appendChild(error);

    return {
        tr: tr,
        value: value,
        pct: pct,
        expected: expected,
        error: error,
    }
}

// Creates a leaderboard space description table cell
function create_leaderboard_space_cell(elem, sub) {
    // Create description cell
    const td = document.createElement("td");
    td.setAttribute("class", "statlabel");

    // Get space description
    let desc;

    if (sub == 2) {
        desc = "Just Visiting";
    } else {
        desc = space_code_to_description(space_codes[elem], true);
    }

    if (space_codes[elem][0] == 'P') {
        // Property

        // Add text span
        let span = document.createElement("span");
        span.innerHTML = desc;
        td.appendChild(span);

        // Create colour block
        const colour = set_to_colour(space_codes[elem][1]);

        let colour_block = document.createElement("span");
        colour_block.setAttribute("class", "colsample");
        colour_block.setAttribute("style", `background-color: ${colour}`);

        // Add colour block
        td.appendChild(colour_block);
    } else {
        // Add text
        td.innerHTML = desc;
    }

    return td;
}

// Creates a leaderboard plain text description table cell
function create_leaderboard_text_cell(text) {
    // Create text cell
    let td = document.createElement("td");
    td.setAttribute("class", "statlabel");

    td.innerText = text;

    return td;
}

// Round a number to a given number of decimal places
function roundnum(num, dp) {
    let mult = Math.pow(10, dp);
    let result = Math.round((num + Number.EPSILON) * mult) / mult;

    if (result === 0) {
        // Turn negative zero in to positive zero
        return 0
    }

    return result;
}

// Colour the text of an element according to a give error value
function colour_error(elem, error, dp) {
    let rnderr = roundnum(error, dp);

    switch (Math.sign(rnderr)) {
        case -1:
            elem.style.color = "red";
            break;
        case 0:
            elem.style.color = "var(--text-color)";
            break;
        case 1:
            elem.style.color = "green";
            break;
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

    if (paused) {
        worker_exec_stop();
    } else {
        worker_exec_start();
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

    if (split_just_visiting) {
        btn.innerText = "Combine Just Visiting";
    } else {
        btn.innerText = "Split Just Visiting";
    }

    let elem;

    // Show/hide percentages on the jail space
    elem = document.getElementById(`pct${space_visit}`);
    elem.style.display = (split_just_visiting ? "none" : "inline");
    elem = document.getElementById(`pct${space_visit}-1`);
    elem.style.display = (split_just_visiting ? "inline" : "none");
    elem = document.getElementById(`pct${space_visit}-2`);
    elem.style.display = (split_just_visiting ? "inline" : "none");

    if (last_stats) {
        // Re-process last stats
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
        btn.innerText = "Top 20 Only";
    } else {
        btn.innerText = "Full Leaderboard";
    }

    if (last_stats) {
        // Re-process last stats
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
