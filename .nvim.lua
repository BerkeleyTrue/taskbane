local dotenv = require("lib.dotenv")

local envs = dotenv.load()

vim.lsp.config('rust_analyzer', {
  cmd_env = {
    DATABASE_URL = envs["DB_URL"]
  },
})
