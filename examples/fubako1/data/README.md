# fubako example1

## How to start the server and add pages

```console
# Please run the following commands in the dev container.

# Start the server
XDG_CONFIG_HOME=/workspaces/fubako/examples/fubako1/xdg_config_home cargo run -- serve

# Open http://localhost:3000 in your browser.

# Add a new page
XDG_CONFIG_HOME=/workspaces/fubako/examples/fubako1/xdg_config_home cargo run -- new

# Edit the created markdown file with your favorite editor.
```

## How to link pages

`[PAGE_ID]`

`PAGE_ID` is ISO8601 basic format in UTC. YYYYMMDDTHHMMSSZ. (e.g. `20251222T214940Z`). `README` is a special page id that is displayed when you visit `/`.

[20251222T214940Z]

Any Markdown link format will probably work.

- [Page 1](20251222T214940Z) `[Page 1](20251222T214940Z)`
- [Page 1][page_1] `[Page 1][page_1]`

Backlinks are automatically detected.

[page_1]: 20251222T214940Z
