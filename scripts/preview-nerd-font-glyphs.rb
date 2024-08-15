#!/usr/bin/env ruby
require 'yaml'

if ARGV.empty?
  warn "Usage: #{$PROGRAM_NAME} <languages.yaml>"
  exit 1
end

LANGUAGES = YAML.load_file(ARGV[0])

LANGUAGES.entries.each do |language, data|
  glyph = data['nerd-font-glyph']
  puts "#{language}: #{glyph}"
end
