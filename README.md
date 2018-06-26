## Rutie

“The tie between Ruby and Rust.”
[![Build Status](https://travis-ci.org/danielpclark/rutie.svg?branch=master)](https://travis-ci.org/danielpclark/rutie)

This README is under process of a rewrite.

This project is a continuation of [ruby-sys](https://github.com/steveklabnik/ruby-sys/) (licensed MIT) and [ruru](https://github.com/d-unseductable/ruru/) (licensed MIT).  This project follows SemVer rules.


## Migrating from Ruru to Rutie

#### &lt;0.1
For using Rutie versions less than 0.1 the change is simple.  Replace all occurences
of the string `ruru` with `rutie` in your program.  And if you would like to use
`ruby-sys` code from Rutie rather than requiring `ruby-sys` you can change all existing
references to `ruby_sys` to `rutie::rubysys`.

#### 0.1

You will have additional considerations to change like `Error` being removed.  For that; change instances of type `ruru::result::Error` to `rutie::AnyException`.

#### 0.2

Migrated `parse_arguments` from `VM` to `util`.

## LICENSE

MIT LICENSE — see [LICENSE](LICENSE)
