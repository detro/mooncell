workflow "Format, Clippy, Test, Build release" {
  on = "push"
  resolves = ["Quickstart"]
}

action "Quickstart" {
  uses = "icepuma/rust-github-actions/quickstart@master"
}
