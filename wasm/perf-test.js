const start = Date.now();
const { lookup4 } = require("./index.js");
(async function () {
  for (let i = 0; i < process.env["A"]; i++) {
    let ip = [0, 0, 0, 0].map(() => Math.floor(Math.random() * 256)).join(".");
    await lookup4(ip);

    if (ip.startsWith("0")) console.log(ip);
  }
  console.log(Date.now() - start);
  //console.log(process.memoryUsage())
})();
