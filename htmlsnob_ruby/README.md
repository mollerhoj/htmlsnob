<img src="htmlsnob.png" align="right"
     alt="The htmlsnob himself" width="128" height="128">

_"A gentleman writes their HTML by hand"_
# HTMLsnob Ruby

Validates and formats HTML and ERB files according to your coding standards.

## Installation

Install the gem and add to the application's Gemfile by executing:
```
    bundle add htmlsnob_ruby
```
If bundler is not being used to manage dependencies, install the gem by executing:
```
    gem install htmlsnob_ruby
```
Or manually add it to your application's Gemfile:
```ruby
    gem 'htmlsnob_ruby'
```

## Usage

To ensure that your erb files are compliant, setup a test file like this:

Minitest:
```ruby
class HtmlSnobTest < Minitest::Test
  include HTMLsnob::Minitest

  def test_htmlsnob
    paths = ['*/*.html.erb', '*/**/*.html.erb'] # Glob patterns to your html or erb files

    config_file = 'htmlsnob_config.yml' # Path to your config file
    (status_code, message) = HtmlsnobRuby.run(paths)

    # Optionally, you can specify the config file like this:
    # (status_code, message) = HtmlsnobRuby.run(paths, config: "htmlsnob_config.yml")

    # If there are any violations, print the message
    if status_code != 0
      puts message
    end

    assert_equal 0, status_code
  end
end
```

rspec:
```ruby
RSpec.describe 'HTMLsnob' do
  it 'validates HTML files' do
    paths = ['*/*.html.erb', '*/**/*.html.erb'] # Glob patterns to your html or erb files

    config_file = 'htmlsnob_config.yml' # Path to your config file
    (status_code, message) = HtmlsnobRuby.run(paths)

    # Optionally, you can specify the config file like this:
    # (status_code, message) = HtmlsnobRuby.run(paths, config: "htmlsnob_config.yml")

    # If there are any violations, print the message
    if status_code != 0
      puts message
    end

    expect(status_code).to eq(0)
  end
end
```
