#!/usr/bin/env bash

if [ -z "$OCKAM_ROOT" ]
then
  echo "Please set the OCKAM_ROOT environment variable to the ockam repository root directory."
  exit 0
fi

perl -ne 'if(/^version = "(\d+)\.(\d+)\.(\d+)(?:-\w)?"/) { $next = $3+1; print "$1.$2.$next"."-dev\n" }' < $OCKAM_ROOT/implementations/rust/ockam/$1/Cargo.toml
