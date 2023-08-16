#!/usr/bin/env ruby
require 'yaml'

if ARGV.empty?
  warn "Usage: #{$PROGRAM_NAME} <languages.yaml>"
  exit 1
end

LANGUAGES = YAML.load_file(ARGV[0])

extensions = Hash.new { |h, k| h[k] = [] }
filenames = Hash.new { |h, k| h[k] = [] }
interpreters = Hash.new { |h, k| h[k] = [] }

LANGUAGES.each do |language, info|
  next if info['heuristics']&.any?

  # TODO: Check for conflicting regexes?
  matchers = info['matchers']
  matchers['extensions']&.each do |ext|
    extensions[ext] << language
  end
  matchers['filenames']&.each do |filename|
    filenames[filename] << language
  end
  matchers['interpreters']&.each do |interpreter|
    interpreters[interpreter] << language
  end
end

extensions.each do |ext, langs|
  puts "Extension #{ext} is used by #{langs.join(', ')}" if langs.size > 1
end

filenames.each do |filename, langs|
  puts "Filename #{filename} is used by #{langs.join(', ')}" if langs.size > 1
end

interpreters.each do |interpreter, langs|
  puts "Interpreter #{interpreter} is used by #{langs.join(', ')}" if langs.size > 1
end
