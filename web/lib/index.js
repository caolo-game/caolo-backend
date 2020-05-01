const bodyParser = require("koa-bodyparser");
const logger = require("koa-logger");
const Router = require("@koa/router");
const Koa = require("koa");
const addon = require("../native");

addon.init();

const compile = (compilationUnit) => {
  return new Promise((resolve, reject) => {
    addon.compile(compilationUnit, (err, res) => {
      if (err) {
        reject(err);
      } else {
        resolve(res);
      }
    });
  });
};

const app = new Koa();

const router = new Router();

router.post("/compile", async (ctx) => {
  const cu = ctx.request.body;
  try {
    const res = await compile(cu);
    ctx.body = res;
  } catch (e) {
    ctx.throw(400, e);
  }
});

app
  .use(logger())
  .use(bodyParser())
  .use(router.routes())
  .use(router.allowedMethods())
  .on("error", (err, ctx) => {
    console.error("server error", err);
  });

const port = process.env.PORT || 8000;

app.listen(port);
