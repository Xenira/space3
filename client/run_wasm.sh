#!/bin/bash

echo "Checking if steam sdk is installed"
if [ ! -d "steam_sdk" ]; then
	echo "Downloading steam sdk"
	mkdir steam_sdk
	cd steam_sdk
	wget -O steamworks_sdk.zip https://partner.steamgames.com/downloads/steamworks_sdk.zip
	unzip "steamworks_sdk.zip"
	rm steamworks_sdk.zip
	cd ..
fi

export STEAM_SDK_LOCATION="$(pwd)/steam_sdk/sdk"
echo "Starting build with sdk $STEAM_SDK_LOCATION"
cargo run --target wasm32-unknown-unknown