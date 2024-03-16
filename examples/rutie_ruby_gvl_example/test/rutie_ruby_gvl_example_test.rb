# frozen_string_literal: true

require 'test_helper'

class RutieRubyExampleTest < Minitest::Test
  def test_it_returns_stack_allocated_captured_variables
    ret = RutieExample.stack_allocated_returning_input
    assert_equal 42, ret
  end

  def test_it_returns_heap_allocated_captured_variables
    ret = RutieExample.heap_allocated_returning_input
    assert_equal 'Object', ret
  end

  def test_it_captures_stack_allocated_variables_and_returns_computation
    ret = RutieExample.stack_allocated_returning_from_closure(5)
    assert_equal '8', ret
  end

  def test_it_captures_heap_allocated_variables_and_returns_computation
    ret = RutieExample.heap_allocated_returning_from_closure(5)
    assert_equal 8, ret
  end

  def test_calls_ruby_method_via__call_with_gvl
    ret = RutieExample.call_ruby_in_call_with_gvl
    assert_equal 'Object', ret
  end

  def test_it_spawns_a_ruby_thread
    ret = RutieExample.create_thread
    assert_equal 'Object', ret
  end
end
