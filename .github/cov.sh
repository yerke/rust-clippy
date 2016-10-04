#!/bin/bash
# Run coverage tests and upload them to coveralls

set -e
wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
tar xzf master.tar.gz
(
    cd kcov-master
    mkdir build
    cd build
    mkdir -p ~/opt/
    cmake -D CMAKE_INSTALL_PREFIX=~/opt/ ..
    make
    make install
)
KCOV_PATH=~/opt/bin/kcov ./util/cov.sh

#set -e
#if [ "$TRAVIS_PULL_REQUEST" == "false" ] &&
#   [ "$TRAVIS_REPO_SLUG" == "Manishearth/rust-clippy" ] &&
#   [ "$TRAVIS_BRANCH" == "master" ] ; then
#    ./util/cov.sh
#
#    kcov --coveralls-id="$TRAVIS_JOB_ID" target/cov 
#fi
