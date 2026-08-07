# Changelog

## [0.2.1](https://github.com/tochka-public/odin_palace/compare/v0.2.0...v0.2.1) (2026-08-07)


### Performance Improvements

* **parser:** accumulate section attributes as borrowed slices ([5052c9b](https://github.com/tochka-public/odin_palace/commit/5052c9b00316c12061b9c6cf365280e604b0166d))

## [0.2.0](https://github.com/tochka-public/odin_palace/compare/v0.1.0...v0.2.0) (2026-08-07)


### Features

* **parser:** cover remaining 1CClientBankExchange 1.01-1.03 document fields ([dbeb12f](https://github.com/tochka-public/odin_palace/commit/dbeb12f7df61724215d3a0fa83f10afe8052ff8b))


### Bug Fixes

* **anonymizer:** correct operator precedence, UTF-8 slicing, dropped lines and value-only purpose replacement ([e8c5ce0](https://github.com/tochka-public/odin_palace/commit/e8c5ce06509844b0d2476dc14ba4c41db2722f8b))
* **parser:** handle UTF-8 BOM, trim attribute values, report Error::Empty for empty input ([349fdb2](https://github.com/tochka-public/odin_palace/commit/349fdb2996f4ed8641471fdea69c8d024d2c8af7))


### Performance Improvements

* **parser:** deserialize sections directly from attribute pairs, stream lines, dedupe intervals in O(log n) ([ef200df](https://github.com/tochka-public/odin_palace/commit/ef200df0b6dd31ad86b72ed8746287796ca80e48))
