
lib = File.expand_path("../lib", __FILE__)
$LOAD_PATH.unshift(lib) unless $LOAD_PATH.include?(lib)
require "rutie_ruby_example/version"

Gem::Specification.new do |spec|
  spec.name          = "rutie_ruby_example"
  spec.version       = RutieRubyExample::VERSION
  spec.authors       = ["Daniel P. Clark"]
  spec.email         = ["6ftdan@gmail.com"]

  spec.summary       = %q{Reverse a string in Rust: a Rutie example extension}
  spec.description   = %q{A Ruby gem whose RutieExample.reverse is written in Rust with Rutie, built and loaded with the rutie gem.}
  spec.homepage      = "https://github.com/danielpclark/rutie/tree/master/examples/rutie_ruby_example"
  spec.license       = "MIT"

  # Specify which files should be added to the gem when it is released.
  # The `git ls-files -z` loads the files in the RubyGem that have been added into git.
  spec.files         = Dir.chdir(File.expand_path('..', __FILE__)) do
    `git ls-files -z`.split("\x0").reject { |f| f.match(%r{^(test|spec|features)/}) }
  end
  spec.bindir        = "exe"
  spec.executables   = spec.files.grep(%r{^exe/}) { |f| File.basename(f) }
  spec.require_paths = ["lib"]

  # Rutie::RakeTask and the one-argument Rutie#init are in rutie 0.0.5. The
  # Gemfile uses the gem in this repository's `gem` submodule.
  spec.add_dependency "rutie", "~> 0.0.5"
  spec.add_development_dependency "rake", ">= 12.0"
  spec.add_development_dependency "minitest", ">= 5.25"
end
