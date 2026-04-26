import { performance } from "node:perf_hooks";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const PROJECT_ROOT = path.resolve(__dirname, "../..");

const DATASETS = [
    ["small", path.join(PROJECT_ROOT, "benchmarks/corpus/small.js")],
    ["medium", path.join(PROJECT_ROOT, "benchmarks/corpus/medium.js")],
    ["large", path.join(PROJECT_ROOT, "benchmarks/corpus/large.js")],
];

const RESULTS_DIR = path.join(PROJECT_ROOT, "benchmarks/results");
const RESULTS_FILE = path.join(RESULTS_DIR, "rslint.csv");

const WARMUP_RUNS = 3;
const MEASURE_RUNS = 20;

const RSLINT_BIN = process.env.RSLINT_BIN ?? "rslint";

function lineCount(filePath) {
    const source = fs.readFileSync(filePath, "utf8");
    return source.split(/\r?\n/).length;
}

function relativeToRoot(filePath) {
    return path.relative(PROJECT_ROOT, filePath).replaceAll(path.sep, "/");
}

function runOnce(filePath) {
    const start = performance.now();

    const result = spawnSync(RSLINT_BIN, [filePath], {
        cwd: PROJECT_ROOT,
        stdio: "ignore",
        shell: process.platform === "win32",
    });

    const end = performance.now();

    if (result.error) {
        throw result.error;
    }

    return {
        totalWallMs: end - start,
        status: result.status,
    };
}

function average(values) {
    return values.reduce((sum, value) => sum + value, 0) / values.length;
}

function benchmarkDataset(name, filePath) {
    if (!fs.existsSync(filePath)) {
        throw new Error(`Dataset file not found: ${filePath}`);
    }

    for (let i = 0; i < WARMUP_RUNS; i++) {
        runOnce(filePath);
    }

    const runs = [];

    for (let i = 0; i < MEASURE_RUNS; i++) {
        runs.push(runOnce(filePath));
    }

    return {
        dataset: name,
        file: relativeToRoot(filePath),
        lines: lineCount(filePath),
        totalWallMs: average(runs.map((run) => run.totalWallMs)),
        lastStatus: runs[runs.length - 1]?.status ?? null,
    };
}

function main() {
    fs.mkdirSync(RESULTS_DIR, { recursive: true });

    const rows = [];

    for (const [name, filePath] of DATASETS) {
        rows.push(benchmarkDataset(name, filePath));
    }

    const csvHeader = [
        "dataset",
        "file",
        "lines",
        "total_wall_ms",
        "last_status",
    ];

    const csvRows = rows.map((row) =>
        [
            row.dataset,
            row.file,
            row.lines,
            row.totalWallMs.toFixed(4),
            row.lastStatus,
        ].join(",")
    );

    const csv = [csvHeader.join(","), ...csvRows].join("\n");

    fs.writeFileSync(RESULTS_FILE, csv, "utf8");

    console.log(csv);
    console.log(`\nSaved results to: ${relativeToRoot(RESULTS_FILE)}`);
}

main();