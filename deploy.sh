#!/bin/bash

~/.cargo/bin/wasm-pack build || exit 1
cd www || exit 1

rm -rf dist

npm run build || exit 1

NAME=ld49

cd dist || exit 1
zip -r ../$NAME ./*
cd ..

scp $NAME.zip necauqua.dev:.

rm $NAME.zip

# shellcheck disable=SC2029 # we want that
ssh necauqua.dev "bash -c 'rm -rf $NAME || exit 1; unzip $NAME.zip -d $NAME; rm -f $NAME.zip'"
