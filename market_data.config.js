const env_production = {
  RUST_LOG: "INFO"
};

module.exports = {
  apps: [
    {
      name: "ticker ETH-PERP",
      script: "./target/release/market_data",
      args: "ftx marketdepth ETH-PERP",
      env_production,
    },
    {
      name: "marketdepth ETH-PERP",
      script: "./target/release/market_data",
      args: "ftx marketdepth ETH-PERP",
      env_production,
    },
    {
      name: "ticker BTC-PERP",
      script: "./target/release/market_data",
      args: "ftx ticker BTC-PERP",
      env_production,
    },
    {
      name: "marketdepth BTC-PERP",
      script: "./target/release/market_data",
      args: "ftx marketdepth BTC-PERP",
      env_production,
    },
  ],
};
