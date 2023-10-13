# frozen_string_literal: true

require 'action_view'
require 'git'
require 'repofetch/config'
require 'repofetch/env'
require 'repofetch/exceptions'
require 'repofetch/plugin'
require 'repofetch/stat'
require 'repofetch/timespan_stat'
require 'repofetch/theme'
require 'repofetch/util'
require 'repofetch/version'

# Main class for repofetch
class Repofetch
  MAX_ASCII_WIDTH = 40
  MAX_ASCII_HEIGHT = 20
  DEFAULT_THEME = Theme.new.freeze
  @plugins = []

  class << self
    attr_accessor :config
    attr_reader :plugins
  end

  # Loads the config, without affecting the file system.
  def self.load_config
    @config = Config.load
  end

  # Loads the config, writing a default config if it doesn't exist.
  def self.load_config!
    @config = Config.load!
  end

  # Registers a plugin.
  #
  # @param [Plugin] plugin The plugin to register
  def self.register_plugin(plugin)
    @plugins << plugin
  end

  # Replaces an existing plugin. If the existing plugin does not exist,
  # then it registers the plugin instead.
  #
  # @param [Plugin] old The plugin to be replaced
  # @param [Plugin] new The new plugin
  def self.replace_or_register_plugin(old, new)
    index = @plugins.find_index(old)
    if index.nil?
      register_plugin(new)
    else
      @plugins[index] = new
      @plugins
    end
  end

  # Returns the plugin that should be used.
  # Raises a +Repofetch::NoPluginsError+ if no plugins are found.
  # Raises a +Repofetch::TooManyPluginsError+ if more than one plugin is found.
  #
  # @param [String] path The path to check.
  # @param [Array<String>] args The arguments passed to the program.
  #
  # @raise [NoPluginsError] If no plugins were selected.
  # @raise [TooManyPluginsError] If more than one plugin was selected.
  #
  # @return [Plugin] A plugin to use.
  def self.get_plugin(path, args)
    path_plugin = get_plugin_for_path(path)
    repo_plugin = get_plugin_for_repo(path)

    raise TooManyPluginsError if path_plugin && repo_plugin

    raise NoPluginsError if path_plugin.nil? && repo_plugin.nil?

    return path_plugin.from_path(path, args) unless path_plugin.nil?

    git = Git.open(path)
    repo_plugin.from_git(git, args)
  end

  # Gets the plugins that matches the given path.
  #
  # @param [String] path The path to check.
  #
  # @return [Array<Plugin>] The plugins that match the path.
  def self.get_plugins_for_path(path)
    @plugins.filter { |plugin_class| plugin_class.matches_path?(path) }
  end

  # Gets a single plugin that matches the given path.
  #
  # @param [String] path The path to check.
  #
  # @return [Plugin, nil] The plugin that matches the path.
  def self.get_plugin_for_path(path)
    plugins = get_plugins_for_path(path)

    raise TooManyPluginsError if plugins.length > 1

    plugins[0]
  end

  # Gets the plugins that matches the given repository.
  #
  # @param [String] path The repository to check.
  #
  # @return [Array<Plugin>] The plugins that match the repository.
  def self.get_plugins_for_repo(path)
    begin
      git = Git.open(path)
    rescue ArgumentError
      return []
    end

    @plugins.filter { |plugin_class| plugin_class.matches_repo?(git) }
  end

  # Gets a single plugin that matches the given repository.
  #
  # @param [String] path The repository to check.
  #
  # @return [Plugin, nil] The plugin that matches the repository.
  def self.get_plugin_for_repo(path)
    plugins = get_plugins_for_repo(path)

    raise TooManyPluginsError if plugins.length > 1

    plugins[0]
  end

  def self.clear_plugins
    @plugins = []
  end
  private_class_method :clear_plugins
end
