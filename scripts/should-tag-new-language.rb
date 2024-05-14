#!/usr/bin/env ruby
# Checks two files. If the new file has exactly one new language compared to
# the old file, and there weren't any removed or renamed languages, then this
# will exit with status 0. Otherwise, it will exit with status 1.
require 'set'
require 'yaml'

if ARGV.length != 2
  warn "Usage: #{$PROGRAM_NAME} <OLD_LANGUAGE_FILE> <NEW_LANGUAGE_FILE>"
  exit 2
end

old_languages_yaml = YAML.load_file(ARGV[0]) || {}
new_languages_yaml = YAML.load_file(ARGV[1]) || {}

old_languages = old_languages_yaml.keys.to_set
new_languages = new_languages_yaml.keys.to_set

removed_languages = old_languages - new_languages
added_languages = new_languages - old_languages

exit_code = removed_languages.size == 0 && added_languages.size == 1 ? 0 : 1
exit exit_code
