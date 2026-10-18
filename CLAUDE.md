# Notes for AI assistants

- `facts.toml` is the single source of truth for key values (limits, device requirements, defaults, dates, estimates). Read it (or the generated `FACTS.md`) before relying on numbers in the docs.
- When you change a value: change `facts.toml`, update every marker `<!--f:ID-->...<!--/f-->` that shows it, run `just facts-index`, then `just check`.
- When you add a new key value to a doc, add a fact and wrap the value in a marker.
- External facts (prices, deadlines, survey numbers, regulations) need `source` and `checked`; re-check them when `just check` warns.
- Keep docs/09-prior-art.md updated with every similar project discussed.
