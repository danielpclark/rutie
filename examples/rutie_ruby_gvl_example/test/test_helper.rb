$LOAD_PATH.unshift File.expand_path("../../lib", __FILE__)
require "rutie_ruby_gvl_example"

require "minitest/autorun"

# The reporter is optional so the tests also run with only minitest installed
# (as in CI).
begin
  require 'color_pound_spec_reporter'
  Minitest::Reporters.use! [ColorPoundSpecReporter.new]
rescue LoadError
end
