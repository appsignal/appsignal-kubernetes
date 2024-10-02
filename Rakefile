BUILDX_NAME = "appsignal-kubernetes-builder".freeze
CROSS_VERSION = "0.2.5".freeze

require_relative "lib/command"

namespace :build do
  namespace :prepare do
    task :buildx do
      name = BUILDX_NAME
      output = Command.run("docker buildx ls", :output => false)
      next if output.include?(name)

      Command.run <<~COMMAND
        docker buildx create \
          --name #{name} \
          --bootstrap \
          --driver=docker-container
      COMMAND
    end

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

  desc "Prepare for builds"
  task :prepare => ["prepare:buildx", "prepare:cross"]
end
