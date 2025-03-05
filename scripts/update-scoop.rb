#!/usr/bin/env ruby
require "json"

if ARGV.length != 2
  warn "Usage: #{$PROGRAM_NAME} <scoop-manifest.json> <tag>"
  exit 1
end

path = ARGV[0]
tag = ARGV[1]
# Make version plain X.Y.Z without "v" prefix
version = tag.delete_prefix("v")

manifest = JSON.load_file(path)
manifest["version"] = version
manifest["url"] = "https://github.com/spenserblack/gengo/releases/download/#{tag}/gengo-x86_64-pc-windows-msvc.tar.gz"
pretty = JSON.pretty_generate(manifest)

File.open(path, "w") do |file|
  file.write(pretty)
  # Final newline
  file.write("\n")
end
