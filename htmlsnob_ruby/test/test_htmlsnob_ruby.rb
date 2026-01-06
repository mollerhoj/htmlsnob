# frozen_string_literal: true

require "test_helper"

class TestHtmlsnobRuby < Minitest::Test
  def test_that_it_has_a_version_number
    refute_nil ::HtmlsnobRuby::VERSION
  end

  def test_no_options_given
    paths = ["test/fixtures/valid.html"]
    (status_code, message) = HtmlsnobRuby.run(paths)
    assert_equal "Success: No issues found\n", message
    assert_equal 0, status_code
  end

  def test_valid_html
    paths = ["test/fixtures/valid.html"]
    config_file = "test/fixtures/config.toml"
    (status_code, message) = HtmlsnobRuby.run(paths, config: config_file)
    assert_equal "Success: No issues found\n", message
    assert_equal 0, status_code
  end

  def test_missing_close_tag
    paths = ["test/fixtures/missing_close_tag.html"]
    config_file = "test/fixtures/config.toml"
    (status_code, message) = HtmlsnobRuby.run(paths, config: config_file)

    expected = <<-EXPECTED
test/fixtures/missing_close_tag.html:
0: <p>Hello World!
   --- Open tag `p` is missing close tag

    EXPECTED

    assert_equal expected, message
    assert_equal 1, status_code
  end

  def test_no_issues_if_empty_config
    paths = ["test/fixtures/missing_close_tag.html"]
    config_file = "test/fixtures/empty_config.toml"
    (status_code, message) = HtmlsnobRuby.run(paths, config: config_file)
    assert_equal "Success: No issues found\n", message
    assert_equal 0, status_code
  end

  def test_yaml_config
    paths = ["test/fixtures/valid.html"]
    config_file = "test/fixtures/config.yml"
    (status_code, message) = HtmlsnobRuby.run(paths, config: config_file)
    assert_equal "Success: No issues found\n", message
    assert_equal 0, status_code
  end

  def test_missing_close_tag_yml
    paths = ["test/fixtures/missing_close_tag.html"]
    config_file = "test/fixtures/config.yml"
    (status_code, message) = HtmlsnobRuby.run(paths, config: config_file)

    expected = <<-EXPECTED
test/fixtures/missing_close_tag.html:
0: <p>Hello World!
   --- Open tag `p` is missing close tag

    EXPECTED

    assert_equal expected, message
    assert_equal 1, status_code
  end
end
