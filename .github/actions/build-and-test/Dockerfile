FROM rust:1.31.1

LABEL "com.github.actions.name"="build-and-test"
LABEL "com.github.actions.description"="cargo build && cargo test"
LABEL "com.github.actions.icon"="play-circle"
LABEL "com.github.actions.color"="gray-dark"

LABEL "repository"="https://github.com/detro/mooncell"
LABEL "maintainer"="Ivan De Marino <detronizator@gmail.com>"

COPY entrypoint.sh /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]