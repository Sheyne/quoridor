const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
  entry: "./bootstrap.js",
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "bootstrap.js",
  },
  experiments: {
    syncWebAssembly: true
  },
  module: {
    rules: [
      {
        test: /\.worker\.js$/,
        use: { loader: "worker-loader" },
      },
    ],
  },
  mode: "production",
  plugins: [
    new CopyWebpackPlugin({
      "patterns": ['index.html', "style.css"]
    })
  ],
  devtool: 'source-map',
};
