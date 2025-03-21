$stdout.sync = true

DOCKER_IMAGE_NAME = "appsignal/appsignal-kubernetes".freeze
BUILDX_NAME = "appsignal-kubernetes-builder".freeze
RELEASE_DIR = "release".freeze
EXECUTABLE_NAME = "appsignal-kubernetes".freeze
CROSS_VERSION = "0.2.4".freeze

TARGETS = {
  "x86_64-unknown-linux-musl" => {
    :release_triple => "amd64-unknown-linux-musl",
    :docker_platform => "linux/amd64"
  },
  "aarch64-unknown-linux-musl" => {
    :release_triple => "arm64-unknown-linux-musl",
    :docker_platform => "linux/arm64"
  }
}.freeze

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

      Command.run("cargo install cross --locked --version #{CROSS_VERSION}")
    end
  end

  desc "Prepare for builds"
  task :prepare => [:cleanup, "prepare:buildx", "prepare:cross"]

  desc "Clean up release artifacts"
  task :cleanup do
    FileUtils.rm_rf(RELEASE_DIR)
    FileUtils.mkdir(RELEASE_DIR)
  end

  task :remove_buildx do
    Command.run("docker buildx rm appsignal-container")
  end

  namespace :target do
    TARGETS.each do |target_triple, config|
      desc "Build #{target_triple} release artifact"
      task target_triple => :prepare do
        target_dir = "tmp/build/#{target_triple}"
        release_dir = File.join(RELEASE_DIR, config[:release_triple])
        FileUtils.mkdir_p(release_dir)
        env = {
          # Make sure to always build on amd64 images.
          # Cross and Docker don't always communicate well and it tries to
          # fetch arm64 images on ARM hosts.
          "CROSS_CONTAINER_OPTS" => "--platform linux/amd64",
          # Tell some dependencies cross compilation is allowed
          "PKG_CONFIG_ALLOW_CROSS" => "1",
          # Point to separate target directory so the two builds are isolated
          "CARGO_TARGET_DIR" => target_dir
        }
        # Build the release artifact for the target triple
        Command.run("cross build --release --target #{target_triple}", :env => env)
        # Copy the release artifact to the release directory
        FileUtils.copy(
          File.join(target_dir, target_triple, "release", EXECUTABLE_NAME),
          File.join(release_dir, EXECUTABLE_NAME)
        )
      end
    end

    desc "Build all release artifacts"
    task :all => TARGETS.keys
  end

  desc "Build local image"
  task :image => "build:target:all" do
    tag = "#{DOCKER_IMAGE_NAME}:local"
    platforms = TARGETS.values.map { |config| config[:docker_platform] }
    options = [
      "--builder=#{BUILDX_NAME}",
      "--file=Dockerfile",
      "--platform=#{platforms.join(",")}",
      "--tag #{tag}",
      "--load"
    ]
    Command.run("docker buildx build #{options.join(" ")} .")

    puts
    puts "Built image '#{tag}'"
  end
end

desc "Publish the Docker image"
task :publish => "build:target:all" do
  tag = "#{DOCKER_IMAGE_NAME}:#{current_version}"
  platforms = TARGETS.values.map { |config| config[:docker_platform] }
  options = [
    "--builder=#{BUILDX_NAME}",
    "--file=Dockerfile",
    "--platform=#{platforms.join(",")}",
    "--tag #{DOCKER_IMAGE_NAME}:latest",
    "--tag #{tag}"
  ]
  options << "--push" unless ENV["PUBLISH_DRY_RUN"]
  Command.run("docker buildx build #{options.join(" ")} .")

  puts
  puts "Published images '#{tag}' and 'latest'"
end

desc "Regenerate the protocol"
task :protocol do
  `mkdir -p protocol`
  `protoc -I ../appsignal-protocol --rust_out=protocol ../appsignal-protocol/kubernetes.proto`
end

def current_version
  Command.run("script/read_version", :output => false).strip
end
