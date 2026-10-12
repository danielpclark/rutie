
lib = File.expand_path("../lib", __FILE__)
$LOAD_PATH.unshift(lib) unless $LOAD_PATH.include?(lib)
require "rutie_derive_example/version"

Gem::Specification.new do |spec|
  spec.name          = "rutie_derive_example"
  spec.version       = RutieDeriveExample::VERSION
  spec.authors       = ["Daniel P. Clark"]
  spec.email         = ["6ftdan@gmail.com"]

  spec.summary       = %q{Text statistics in Rust: a Rutie derive example extension}
  spec.description   = %q{A Ruby gem whose TextStats module and Document class are written in Rust with Rutie's #[rubyclass]/#[rubymethods] attributes, built and loaded with the rutie gem.}
  spec.homepage      = "https://github.com/danielpclark/rutie/tree/master/examples/rutie_derive_example"
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
