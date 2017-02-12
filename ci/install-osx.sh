#!/bin/sh -e

brew update
brew install llvm@3.8

if [[ "$CXX" == "clang++" ]]; then
  brew unlink gcc
  export CXX=$(which clang++-3.8)
elif [[ "$CXX" == "g++" ]]; then
  export CXX=$(which g++)
else
  exit 1
fi

SL=/System/Library
PL=com.apple.ReportCrash
launchctl unload -w ${SL}/LaunchAgents/${PL}.plist
sudo launchctl unload -w ${SL}/LaunchDaemons/${PL}.Root.plist
