#!/bin/sh

cp -r $TRUNK_SOURCE_DIR/assets $TRUNK_STAGING_DIR/assets

echo "Removing .txt files from $TRUNK_STAGING_DIR"
find $TRUNK_STAGING_DIR -name "*.txt" -type f -delete -print

echo "Removing .json files from $TRUNK_STAGING_DIR"
find $TRUNK_STAGING_DIR -name "*.json" -type f -delete -print

echo "Removing .aseprite files from $TRUNK_STAGING_DIR"
find $TRUNK_STAGING_DIR -name "*.aseprite" -type f -delete -print

echo "Removing .xcf files from $TRUNK_STAGING_DIR"
find $TRUNK_STAGING_DIR -name "*.xcf" -type f -delete -print