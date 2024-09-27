CROSS_VERSION = "0.2.5".freeze

require_relative "lib/command"

namespace :build do
  namespace :prepare do
    task :cross do
      output =
        begin
          Command.run("cross --version", :output => false)
        rescue Command::CommandFailedError
          ""
        end
      next if output.include?("cross #{CROSS_VERSION}")

      Command.run("cargo install cross --version #{CROSS_VERSION}")
    end
  end
end
