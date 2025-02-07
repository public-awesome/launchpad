rust_version = "rust:1.83"

def main(ctx):
    return [
        pipeline_test_and_build(ctx)
    ]


def pipeline_test_and_build(ctx):
    return {
    "kind": "pipeline",
    "type": "docker",
    "name": "test_and_build",
    "steps": [
      step_fetch(ctx),
      cargo_test_all(ctx),
      cargo_lint(ctx),
      cargo_wasm_build(ctx)
    ],
  }

# Fetch the latest tags from the repository
def step_fetch(ctx):
    return {
        "name": "fetch",
        "image": "alpine/git",
        "commands": [
            "git fetch --tags"
        ]
    }

def cargo_test_all(ctx):
    return {
        "name": "test",
        "image": rust_version,
        "commands": ["cargo test --locked"]
    }

def cargo_lint(ctx):
    return {
        "name": "lint",
        "image": rust_version,
        "commands": [
            "rustup component add rustfmt",
            "rustup component add clippy",
            "cargo fmt -- --check", 
            "cargo clippy --all-targets -- -D warnings"
        ]
    }

def cargo_wasm_build(ctx):
    return {
        "name": "wasm_build",
        "image": rust_version,
        "commands": ["sh scripts/wasm_build.sh"]
    }
