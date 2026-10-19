# Design repository tasks. Sibling repositories (../weft-verifier, ../weft-parts, ../weft-boardchecker, ../weft-website) are used for code checks.

factcheck := "cargo run --quiet --locked --manifest-path tools/factcheck/Cargo.toml --"

# Check docs and code against facts.toml.
check:
    {{factcheck}} check

# Regenerate FACTS.md from facts.toml.
facts-index:
    {{factcheck}} index

# Lint the checker itself.
lint:
    cargo fmt --manifest-path tools/factcheck/Cargo.toml --check
    cargo clippy --locked --manifest-path tools/factcheck/Cargo.toml -- -D warnings

# Validates requirements/*.sdoc and renders HTML and ReqIF to target/requirements.
requirements:
    strictdoc export . --formats=html,reqif-sdoc --output-dir target/requirements

ci: lint check requirements
