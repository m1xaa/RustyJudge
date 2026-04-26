import { ESLint } from "eslint";
import { performance } from "node:perf_hooks";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const PROJECT_ROOT = path.resolve(__dirname, "../..");

const ESLINT_CONFIG_FILE = path.join(__dirname, "eslint.config.mjs");

const DATASETS = [
    ["small", path.join(PROJECT_ROOT, "benchmarks/corpus/small.js")],
    ["medium", path.join(PROJECT_ROOT, "benchmarks/corpus/medium.js")],
    ["large", path.join(PROJECT_ROOT, "benchmarks/corpus/large.js")],
];

const RESULTS_DIR = path.join(PROJECT_ROOT, "benchmarks/results");
const RESULTS_FILE = path.join(RESULTS_DIR, "eslint.csv");

const WARMUP_RUNS = 3;
const MEASURE_RUNS = 20;

function lineCount(filePath) {
    const source = fs.readFileSync(filePath, "utf8");
    return source.split(/\r?\n/).length;
}

function relativeToRoot(filePath) {
    return path.relative(PROJECT_ROOT, filePath).replaceAll(path.sep, "/");
}

function sumStats(results) {
    let parseMs = 0;
    let ruleMs = 0;
    let eslintInternalTotalMs = 0;
    let diagnostics = 0;

    for (const result of results) {
        diagnostics += result.messages.length;

        const passes = result.stats?.times?.passes ?? [];

        for (const pass of passes) {
            parseMs += pass.parse?.total ?? 0;
            eslintInternalTotalMs += pass.total ?? 0;

            const rules = pass.rules ?? {};
            for (const ruleTiming of Object.values(rules)) {
                ruleMs += ruleTiming.total ?? 0;
            }
        }
    }

    return {
        parseMs,
        ruleMs,
        eslintInternalTotalMs,
        diagnostics,
    };
}

async function runOnce(eslint, filePath) {
    const start = performance.now();

    const results = await eslint.lintFiles([filePath]);

    const end = performance.now();

    const stats = sumStats(results);

    return {
        totalWallMs: end - start,
        ...stats,
    };
}

function average(values) {
    return values.reduce((sum, value) => sum + value, 0) / values.length;
}

async function benchmarkDataset(name, filePath) {
    if (!fs.existsSync(filePath)) {
        throw new Error(`Dataset file not found: ${filePath}`);
    }

    if (!fs.existsSync(ESLINT_CONFIG_FILE)) {
        throw new Error(`ESLint config file not found: ${ESLINT_CONFIG_FILE}`);
    }

    const eslint = new ESLint({
        cwd: PROJECT_ROOT,
        overrideConfigFile: ESLINT_CONFIG_FILE,
        stats: true,
        cache: false,
        concurrency: "off",
        errorOnUnmatchedPattern: true,
    });

    for (let i = 0; i < WARMUP_RUNS; i++) {
        await runOnce(eslint, filePath);
    }

    const runs = [];

    for (let i = 0; i < MEASURE_RUNS; i++) {
        runs.push(await runOnce(eslint, filePath));
    }

    return {
        dataset: name,
        file: relativeToRoot(filePath),
        lines: lineCount(filePath),
        parseMs: average(runs.map((run) => run.parseMs)),
        ruleMs: average(runs.map((run) => run.ruleMs)),
        eslintInternalTotalMs: average(runs.map((run) => run.eslintInternalTotalMs)),
        totalWallMs: average(runs.map((run) => run.totalWallMs)),
        diagnostics: Math.round(average(runs.map((run) => run.diagnostics))),
    };
}

async function main() {
    fs.mkdirSync(RESULTS_DIR, { recursive: true });

    const rows = [];

    for (const [name, filePath] of DATASETS) {
        rows.push(await benchmarkDataset(name, filePath));
    }

    const csvHeader = [
        "dataset",
        "file",
        "lines",
        "parse_ms",
        "rule_ms",
        "eslint_internal_total_ms",
        "total_wall_ms",
        "diagnostics",
    ];

    const csvRows = rows.map((row) =>
        [
            row.dataset,
            row.file,
            row.lines,
            row.parseMs.toFixed(4),
            row.ruleMs.toFixed(4),
            row.eslintInternalTotalMs.toFixed(4),
            row.totalWallMs.toFixed(4),
            row.diagnostics,
        ].join(",")
    );

    const csv = [csvHeader.join(","), ...csvRows].join("\n");

    fs.writeFileSync(RESULTS_FILE, csv, "utf8");

    console.log(csv);
    console.log(`\nSaved results to: ${relativeToRoot(RESULTS_FILE)}`);
}

main().catch((error) => {
    console.error(error);
    process.exit(1);
});