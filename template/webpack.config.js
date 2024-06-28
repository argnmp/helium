const path = require('path');

module.exports = {
  entry: {
    entry: './src/app/entry.js',
    module: './src/app/index.js',
  },
  output: {
    path: path.resolve(__dirname, 'dist', 'static'),
    filename: '[name]_bundle.js',
  },
  mode: 'production',
};
