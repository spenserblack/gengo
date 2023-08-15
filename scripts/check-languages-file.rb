#!/usr/bin/env ruby
require 'yaml'

def sorted?(array)
  sorted = true
  array.each_cons(2) do |a, b|
    if a > b
      sorted = false
      yield a, b if block_given?
    end
  end
  sorted
end

PROJECT_ROOT = File.expand_path(File.join(File.dirname(__FILE__), '..'))
LANGUAGES_FILEPATH = File.expand_path(File.join(PROJECT_ROOT, './gengo/languages.yaml'))
LANGUAGES = YAML.load_file(LANGUAGES_FILEPATH)

exit_code = 0
sorted?(LANGUAGES.keys) do |lang1, lang2|
  STDERR.puts "Language '#{lang1}' and '#{lang2}' are out of order"
  exit_code = 1
end

LANGUAGES.each do |langname, langdef|
  if !langdef.is_a?(Hash)
    STDERR.puts "#{langname}: must be an object"
    exit_code = 1
    next
  end
  missing_required = ['category', 'color', 'matchers'].any? do |key|
    if langdef.nil? || !langdef.has_key?(key)
      STDERR.puts "#{langname}: missing required key '#{key}'"
      exit_code = 1
      true
    end
  end
  next if missing_required

  if langdef.key?('heuristics') && !langdef['heuristics'].is_a?(Array)
    STDERR.puts "#{langname}: 'heuristics' must be an array"
    exit_code = 1
  end
  heuristics = langdef['heuristics'] || []
  sorted?(heuristics) do |a, b|
    STDERR.puts "#{langname}: 'heuristics' are out of order: '#{a}' and '#{b}'"
    exit_code = 1
  end

  bad_matchers = false
  matchers = langdef['matchers']
  matcher_keys = ['extensions', 'filenames', 'interpreters', 'patterns']
  if !matchers.is_a?(Hash)
    STDERR.puts "#{langname}: 'matchers' must be an object"
    exit_code = 1
    bad_matchers = true
  else
    matcher_keys.each do |key|
      if matchers.has_key?(key) && !matchers[key].is_a?(Array)
        STDERR.puts "#{langname}: 'matchers.#{key}' must be an array"
        exit_code = 1
        bad_matchers = true
      end
    end
  end
  next if bad_matchers

  unless matcher_keys.any? { |key| matchers.has_key?(key) }
    STDERR.puts "#{langname}: 'matchers' must have at least one of #{matcher_keys.join(', ')}"
    exit_code = 1
  end

  sorted?(matchers.keys) do |a, b|
    STDERR.puts "#{langname}: 'matchers' are out of order: '#{a}' and '#{b}'"
    exit_code = 1
  end

  matchers.each do |matcher_name, matcher|
    unless matcher.is_a?(Array)
      STDERR.puts "#{langname}: matcher '#{matcher_name}' must be an array"
      exit_code = 1
      next
    end
    sorted?(matcher) do |a, b|
      STDERR.puts "#{langname}: matcher '#{matcher_name}' is out of order: '#{a}' and '#{b}'"
      exit_code = 1
    end
  end

  if langdef.key?('priority') && !langdef['priority'].is_a?(Integer) && langdef['priority'].between?(0, 100)
    STDERR.puts "#{langname}: 'priority' must be an integer between 0 and 100"
    exit_code = 1
  end
end

exit exit_code
