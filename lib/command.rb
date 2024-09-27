module Command
  module_function

  def run(command, env: {}, output: true)
    read, write = IO.pipe
    options = { [:out, :err] => write }

    pid = spawn(env, "set -eux; #{command}", options)
    output_lines = []
    thread =
      Thread.new do
        # Read the output as it's written
        # Store it for later and output it (if `output` is true)
        while line = read.readline # rubocop:disable Lint/AssignmentInCondition
          # Output lines as the program runs
          puts line if output
          # Store the output for later
          output_lines << line
        end
      rescue EOFError
        # Do nothing, nothing to read anymore
      end
    _pid, status = Process.wait2(pid)
    write.close
    thread.join
    cmd_output = output_lines.join
    raise CommandFailedError.new(command, cmd_output) unless status.success?

    cmd_output
  end

  class CommandFailedError < StandardError
    def initialize(command, output)
      @command = command
      @output = output
      super()
    end

    def message
      "The command has failed to run: #{@command}\nOutput:\n#{@output}"
    end
  end
end
