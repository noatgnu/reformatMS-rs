# This script takes care of testing your crate

set -ex

# TODO This is the "test phase", tweak it as you see fit
main() {
    cross build --target $TARGET
    cross build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET
    cross test --target $TARGET --release

    cross run --target $TARGET -- -i Ions.csv -f FDR.csv -o out.csv -s true
    cross run --target $TARGET --release -- -i Ions.csv -f FDR.csv -o out.csv -s true
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
