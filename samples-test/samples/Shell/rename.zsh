#!/bin/zsh

# Check if correct number of arguments is provided
if (( $# != 2 )); then
    print "Usage: $0 old_filename new_filename"
    exit 1
fi

# Check if the source file exists
if [[ ! -f $1 ]]; then
    print "Error: File '$1' does not exist"
    exit 1
fi

# Rename the file
mv $1 $2

if (( $? == 0 )); then
    print "File successfully renamed from '$1' to '$2'"
else
    print "Error: Failed to rename the file"
fi
