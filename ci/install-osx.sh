#!/bin/sh -e

brew update
brew unlink gcc
brew install homebrew/versions/llvm38
export CXX=$(which clang++-3.8)
export LLVM_CONFIG=$(which llvm-config-3.8)
SL=/System/Library
PL=com.apple.ReportCrash
launchctl unload -w ${SL}/LaunchAgents/${PL}.plist
sudo launchctl unload -w ${SL}/LaunchDaemons/${PL}.Root.plist
