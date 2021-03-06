const env_production = {
  RUST_LOG: "WARN",
  ENV: "sim"
};

module.exports = {
  apps: [
    {
      name: "marketdepth-ftx-btcperp",
      script: "./target/release/market_data",
      args: "ftx marketdepth BTC-PERP",
      env_production,
    },
    {
      name: "marketdepth-ftx-ethperp",
      script: "./target/release/market_data",
      args: "ftx marketdepth ETH-PERP",
      env_production,
    },
    {
      name: "latency-mm",
      script: "./target/release/container",
      args: "latency-mm",
      env_production,
    },
  ],
};
