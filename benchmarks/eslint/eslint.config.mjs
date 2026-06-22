export default [
    {
        files: ["benchmarks/corpus/**/*.js"],
        languageOptions: {
            ecmaVersion: 2022,
            sourceType: "script"
        },
        rules: {
            "no-var": "warn",
            "no-unused-vars": "warn",
            "eqeqeq": "warn",
            "no-console": "warn",
            "no-dupe-args": "warn",
            "no-empty": "warn",
            "no-unreachable": "warn",
            "consistent-return": "warn",
            "no-self-assign": "warn"
        }
    }
];