function is_rust(p)
  if p:find("target") ~= nil then return false end
  return p:ext() == "rs" or p:ext() == "toml"
end

return {
  {
    should_run = is_rust,
    redirect_stderr = "/tmp/cargo.err",
    commands = {
      -- {
        -- name = "Running cargo check",
        -- command = "cargo check --release --color=always",
      -- },
      {
        name = "Running cargo build",
        command = "cargo build --release --color=always",
      },
      -- {
        -- name = "Running cargo clippy",
        -- command = "cargo clippy --color=always -- -D warnings",
      -- },
    }
  },
}
