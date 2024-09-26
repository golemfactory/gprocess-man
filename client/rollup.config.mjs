import commonjs from "@rollup/plugin-commonjs";
import typescript from "@rollup/plugin-typescript";
import nodeResolve from "@rollup/plugin-node-resolve";

export default {
    input: "src/browser.ts",
    output: {
        file: "lib/browser.js",
    },
    plugins: [
        commonjs(),
        typescript({ compilerOptions: { module: "es6" } }),
        nodeResolve(),
    ]
};
