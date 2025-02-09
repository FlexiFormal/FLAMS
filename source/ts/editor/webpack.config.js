/* --------------------------------------------------------------------------------------------
 * Copyright (c) 2024 TypeFox and others.
 * Licensed under the MIT License. See LICENSE in the package root for license information.
 * ------------------------------------------------------------------------------------------ */

// solve: __dirname is not defined in ES module scope
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
//const TerserPlugin = require('terser-webpack-plugin');

const config = {
    mode: 'development',//'production',
    entry: resolve(__dirname, 'src', 'flams_editor.ts'),
    module: {
        rules: [
            {
                test: /\.css$/,
                use: ['style-loader', 'css-loader']
            },
            {
                test: /\.ts?$/,
                use: ['ts-loader']
            },
            {
              resourceQuery: /raw/,
              type: 'asset/source',
            }
        ]
    },
    experiments: {
        outputModule: true
    },
    output: {
        filename: 'flams_editor.js',
        path: resolve(__dirname, 'dist'),
        module: true,
        workerChunkLoading: 'import',
        environment: {
            dynamicImportInWorker: true
        },
        library: {
            type: 'module'
        }
    },
    target: 'web',
    resolve: {
        extensions: ['.ts', '.js', '.json', '.ttf']
    },
    devtool: 'source-map'
};

export default config;
