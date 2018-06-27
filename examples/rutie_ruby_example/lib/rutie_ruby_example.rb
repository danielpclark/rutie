require 'rutie_ruby_example/version'
require 'fiddle'

module RutieRubyExample
  module Platform
    class << self
      def ffi_library
        file = [lib_prefix,'rutie_ruby_example.',lib_suffix]

        File.join(rust_release, file.join())
      end

      def operating_system
        case host_os()
        when /linux|bsd|solaris/ then 'linux'
        when /darwin/ then 'darwin'
        when /mingw|mswin/ then 'windows'
        else host_os()
        end
      end

      def lib_prefix
        case operating_system()
        when /windows/ then ''
        when /cygwin/ then 'cyg'
        else 'lib'
        end
      end

      def lib_suffix
        case operating_system()
        when /darwin/ then 'dylib'
        when /linux/ then 'so'
        when /windows|cygwin/ then 'dll'
        else 'so'
        end
      end

      def rust_release
        File.expand_path('../target/release/', __dir__)
      end

      def host_os
        RbConfig::CONFIG['host_os'].downcase
      end
    end
  end

  LIBRARY = Platform.ffi_library()
  Fiddle::Function.
    new(Fiddle.dlopen(LIBRARY)['Init_rutie_ruby_example'], [], Fiddle::TYPE_VOIDP).
    call
end
