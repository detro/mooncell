workflow "On Push: build and test" {
  on = "push"
  resolves = ["Build and Test"]
}

action "Build and Test" {
  uses = "detro/mooncell/.github/actions/build-and-test@master"
}
