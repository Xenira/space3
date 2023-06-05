#!/bin/sh

echo "Removing .txt files from $TRUNK_STAGING_DIR"
find $TRUNK_STAGING_DIR -name "*.txt" -type f -delete

echo "Removing .aseprite files from $TRUNK_STAGING_DIR"
find $TRUNK_STAGING_DIR -name "*.aseprite" -type f -delete

echo "Removing .xcf files from $TRUNK_STAGING_DIR"
find $TRUNK_STAGING_DIR -name "*.xcf" -type f -delete