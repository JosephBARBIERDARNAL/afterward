require "yaml"

yaml_files = Dir.glob("src/assets/**/*.{yaml,yml}").sort
abort "No game data YAML files found" if yaml_files.empty?

yaml_files.each do |path|
  YAML.parse_file(path)
rescue Psych::SyntaxError => error
  warn "#{path}: #{error.message}"
  exit 1
end

puts "Validated #{yaml_files.length} game data YAML files"
