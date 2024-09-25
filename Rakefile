DOCKER_IMAGE_NAME = "appsignal/kubernetes-build".freeze
BUILD_DOCKER_IMAGE_NAME = "appsignal/kubernetes-build".freeze
BUILDX_NAME = "appsignal-kubernetes-builder".freeze
RELEASE_DIR = "release".freeze
EXECUTABLE_NAME = "appsignal-kubernetes".freeze

module CommandHelper
  module_function

  def run(command)
    output = `set -eux; #{command}`
    status = Process.last_status
    unless status.success?
      puts "Command failed: #{command}"
      exit 1
    end
    output
  end
end

class DockerHelper
  class << self
    def setup_buildx
      output = CommandHelper.run("docker buildx ls")
      if output.include?(BUILDX_NAME)
        return
      end

      CommandHelper.run(
        "docker buildx create --name #{BUILDX_NAME} --bootstrap --driver=docker-container"
      )
    end

    def build_image(name:, dockerfile:, platform:)
      CommandHelper.run <<~COMMAND
        docker build \
          -f #{dockerfile} \
          --platform #{platform} \
          -t #{name} \
          .
      COMMAND
    end

    def run(command, image:, platform:, options: [])
      CommandHelper.run <<~COMMAND
        docker run \
          --rm \
          --platform=#{platform} \
          #{map_cli_options(options)} \
          #{image} \
          #{command}
      COMMAND
    end

    private

    def map_cli_options(options)
      options.map { |key, value| "--#{key}=#{value}" }.join(" ")
    end
  end
end

class PublishHelper
  def self.current_version
    CommandHelper.run("git describe --tags --abbrev=0 | tr -d v").strip
  end
end

namespace :build do
  task :prepare do
    FileUtils.rm_rf(RELEASE_DIR)
    FileUtils.mkdir(RELEASE_DIR)
    DockerHelper.setup_buildx
    DockerHelper.build_image(
      :name => "#{BUILD_DOCKER_IMAGE_NAME}:build-amd64",
      :dockerfile => "Dockerfile.build",
      :platform => "linux/amd64"
    )
    DockerHelper.build_image(
      :name => "#{BUILD_DOCKER_IMAGE_NAME}:build-arm64",
      :dockerfile => "Dockerfile.build",
      :platform => "linux/arm64"
    )
  end

  task :all => :prepare do
    DockerHelper.run(
      "cargo build --release --target x86_64-unknown-linux-musl",
      :image => "#{BUILD_DOCKER_IMAGE_NAME}:build-amd64",
      :platform => "linux/amd64",
      :options => {
        :volume => "'#{Dir.pwd}':/project",
        :workdir => "/project",
      }
    )
    release_dir = File.join(RELEASE_DIR, "amd64-unknown-linux-musl")
    FileUtils.mkdir_p(release_dir)
    FileUtils.copy(
      "target/x86_64-unknown-linux-musl/release/#{EXECUTABLE_NAME}",
      File.join(release_dir, EXECUTABLE_NAME)
    )

    DockerHelper.run(
      "cargo build --release --target aarch64-unknown-linux-musl",
      :image => "#{BUILD_DOCKER_IMAGE_NAME}:build-arm64",
      :platform => "linux/arm64",
      :options => {
        :volume => "'#{Dir.pwd}':/project",
        :workdir => "/project",
      }
    )
    release_dir = File.join(RELEASE_DIR, "arm64-unknown-linux-musl")
    FileUtils.mkdir_p(release_dir)
    FileUtils.copy(
      "target/aarch64-unknown-linux-musl/release/#{EXECUTABLE_NAME}",
      File.join(release_dir, EXECUTABLE_NAME)
    )
  end
end

task :publish => ["build:all"] do
  platforms = ["linux/amd64", "linux/arm64"]
  options = [
    "--tag #{DOCKER_IMAGE_NAME}:latest",
    "--tag #{DOCKER_IMAGE_NAME}:#{PublishHelper.current_version}"
  ]
  options << "--push" unless ENV["PUBLISH_DRY_RUN"]
  CommandHelper.run <<~COMMAND
    docker buildx build \
      --builder=#{BUILDX_NAME} \
      --load \
      --file Dockerfile \
      --platform=#{platforms.join(",")} \
      #{options.join(" ")} \
      .
  COMMAND
end
