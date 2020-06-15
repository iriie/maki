# stolen from https://github.com/Arzte/Arzte-bot/blob/master/ci/before_deploy.sh
# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=$(mktemp -d)

    test -f Cargo.lock || cargo generate-lockfile

    cp target/release/maki $stage/maki
    blake2 $stage/maki > $stage/maki.blake2

    cd $stage
    tar czf $src/maki.tar.gz *
    cd $src

    rm -rf $stage
}

main