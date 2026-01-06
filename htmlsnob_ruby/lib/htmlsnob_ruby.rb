# frozen_string_literal: true

require_relative "htmlsnob_ruby/version"
require_relative "htmlsnob_ruby/htmlsnob_ruby"
require 'yaml'
require 'toml-rb'
require 'tempfile'

module HtmlsnobRuby
  class Error < StandardError; end

  def self.run(paths, options = {})
    config = options[:config]

    if config && (config.end_with?(".yml") || config.end_with?(".yaml"))
      # YAML
      Tempfile.create(['converted_config', '.toml']) do |temp|
        temp.write(TomlRB.dump(YAML.load_file(config)))
        temp.rewind
        HtmlsnobRuby.run_simple(paths, temp.path)
      end
    else 
      # TOML
      HtmlsnobRuby.run_simple(paths, config)
    end
  end
end
