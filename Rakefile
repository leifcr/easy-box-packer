require 'rake'
require 'rspec/core/rake_task'

task :build_rust do
  # Is Rust installed?
  # Determine if the cargo command works
  begin
    cargo_v = `cargo -V`
  rescue Errno::ENOENT
    raise 'Cargo not found. Install it.'
  end

  # Is the Rust version (matching the Cargo version) above a certain level?
  # cargo_version = cargo_v.match(/\Acargo (\d+)\.(\d+)\.(\d+) /)[1..3].map(&:to_i)
  # raise "Too old Cargo (ver. #{cargo_v}). Update it." if (cargo_version <=> [1, 40, 0]).negative?

  # Build Rust
  system "cargo build --release --verbose"

  # Product file name
  # Depends on OS
  lib_name = "rutie_box_packer"
  file_name =
    case RbConfig::CONFIG['host_os'].downcase
    when /darwin/      then "lib#{lib_name}.dylib"
    when /mingw|mswin/ then "#{lib_name}.dll"
    when /cygwin/      then "cyg#{lib_name}.dll"
    else                    "lib#{lib_name}.so"
    end

  # Product lib/Move directly below
  # FileUtils.mv __dir__ + "/../target/release/#{file_name}", __dir__ + "/../lib/"
  # FileUtils.rmtree __dir__ + "/../target/"
end

task :clean_rust do
  system 'cargo clean'
end

RSpec::Core::RakeTask.new(:spec) do |t|
  t.pattern = Dir.glob("spec/**/*_spec.rb")
  t.rspec_opts = "--format documentation"
end
task default: :spec
