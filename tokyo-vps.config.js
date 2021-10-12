const env_production = {
  RUST_LOG: "WARN",
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
      script: "./target/release/luban",
      args: "latency-mm",
      env: {
        ENV: 'sim'
      },
      env_production,
    },
    {
      name: "swap-mm-ethusd",
      script: "./target/release/luban",
      args: "latency-mm",
      env: {
        ENV: '_'
      },
      env_production,
    },
  ],
};
