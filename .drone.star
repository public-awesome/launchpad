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
      cargo_test_all(ctx)
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
        "image": "rust:1.83",
        "commands": ["cargo test --locked"]
    }
