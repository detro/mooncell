workflow "On Push: build and test" {
  on = "push"
  resolves = ["Build and Test"]
}

action "Build and Test" {
  uses = "./.github/actions/build-and-test@master"
}
