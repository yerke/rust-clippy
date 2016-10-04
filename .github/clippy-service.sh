#!/bin/bash
# Trigger rebuild of the clippy-service, to keep it up to date with clippy itself

set -e
if [ "$TRAVIS_PULL_REQUEST" == "false" ] &&
   [ "$TRAVIS_REPO_SLUG" == "Manishearth/rust-clippy" ] &&
   [ "$TRAVIS_BRANCH" == "master" ] &&
   [ "$TRAVIS_TOKEN_CLIPPY_SERVICE" != "" ] ; then

   curl -s -X POST \
        -H "Content-Type: application/json" \
        -H "Accept: application/json" \
        -H "Travis-API-Version: 3" \
        -H "Authorization: token $TRAVIS_TOKEN_CLIPPY_SERVICE" \
        -d "{ \"request\": { \"branch\":\"master\" }}" \
        https://api.travis-ci.org/repo/ligthyear%2Fclippy-service/requests

else
  echo "Ignored"
fi
