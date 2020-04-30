var addon = require("../native");

let cu = {
  nodes: {
    0: { node: { Start: null } },
  },
};

const res = addon.compile(cu);

console.log(JSON.stringify(res, null, 4));
