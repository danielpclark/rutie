require "test_helper"

class RutieDeriveExampleTest < Minitest::Test
  TEXT = "The cat and the hat. The end!"

  def test_constants_and_functions
    assert_equal "0.1.0", TextStats::VERSION
    assert_equal %w[a b c], TextStats.words("A, b... C")
  end

  def test_document
    document = TextStats::Document.new(TEXT, title: "Cats")

    assert_equal "Cats", document.title
    assert_equal 7, document.word_count
    assert_equal [["the", 3], ["and", 1]], document.frequencies(top: 2)
    assert document.include?("HAT")
    assert_equal "cat", TextStats::Document.new("a cat").longest_word
  end

  def test_accessors_and_copies
    document = TextStats::Document.new(TEXT)
    copy = document.dup
    copy.title = "Copy"

    assert_equal "untitled", document.title
    assert_equal "Copy", copy.title
    assert_raises(TypeError) { copy.title = 1 }
  end

  def test_blocks
    seen = []
    count = TextStats::Document.new("one two").each_word { |word, index| seen << [word, index] }

    assert_equal 2, count
    assert_equal [["one", 0], ["two", 1]], seen
    assert_raises(LocalJumpError) { TextStats::Document.new("one").each_word }
  end

  def test_errors
    error = assert_raises(TextStats::EmptyError) { TextStats::Document.new(" ... ") }
    assert_equal "the text has no words", error.message
    assert_kind_of TextStats::Error, error

    error = assert_raises(TextStats::Error) { TextStats::Document.new(TEXT).frequencies(top: 0) }
    assert_equal "top must be at least 1, got 0", error.message

    assert_raises(ArgumentError) { TextStats::Document.new(TEXT, color: "red") }
    assert_raises(TypeError) { TextStats::Document.new(42) }
  end

  def test_ruby_subclass
    subclass = Class.new(TextStats::Document) do
      def initialize(text)
        super(text.upcase, title: "loud")
      end
    end

    document = subclass.new("quiet words")
    assert_equal "loud", document.title
    assert_equal 2, document.word_count
  end

  def test_rbs
    signature = TextStats.rbs

    assert_includes signature, "class Document"
    assert_includes signature, "def word_count: () -> Integer"
    assert_includes signature, "def initialize: (String text, ?title: String) -> void"
    assert_includes signature, "class EmptyError < TextStats::Error"
  end
end
