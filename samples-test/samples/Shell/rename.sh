#!/bin/sh

# Check if correct number of arguments is provided
if [ $# -ne 2 ]; then
    echo "Usage: $0 old_filename new_filename"
    exit 1
fi

# Check if the source file exists
if [ ! -f "$1" ]; then
    echo "Error: File '$1' does not exist"
    exit 1
fi

# Rename the file
mv "$1" "$2"

if [ $? -eq 0 ]; then
    echo "File successfully renamed from '$1' to '$2'"
else
    echo "Error: Failed to rename the file"
fi
