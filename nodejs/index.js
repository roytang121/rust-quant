const redis = require("redis");
const nano = require("nanomsg");
const dayjs = require("dayjs");
const client = redis.createClient({ port: 10400 });

client.subscribe("ftx:eth-perp");

client.on("message", (channel, msg) => {
  const json = JSON.parse(msg);
  const timestamp = parseInt(json.timestamp);
  console.log(`Exchange latency: ${json.latency}`);
  console.log(`Redis latency: ${Date.now() - timestamp}`);
});

const sub = nano.socket('sub');
sub.connect("tcp://127.0.0.1:10404")

sub.on('data', msg => {
  const json = JSON.parse(msg);
  const timestamp = parseInt(json.timestamp);
  // console.log(`Exchange latency: ${json.latency}`);
  console.log(`nng latency: ${Date.now() - timestamp}`);
});