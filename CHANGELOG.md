# Changelog

All notable changes to this project will be documented in this file.

## [0.2.2] - 2026-10-12

### Bug Fixes

- *(client)* Disable native compression so reads work on ClickHouse 26.9 (#7) by @Badabum in [#7](https://github.com/TabularisDB/tabularis-clickhouse-plugin/pull/7)

### Miscellaneous

- Add CI and version suggestion workflows (#6) by @debba in [#6](https://github.com/TabularisDB/tabularis-clickhouse-plugin/pull/6)
## [0.2.1] - 2026-10-11

### Bug Fixes

- *(ci)* Release body carries only the released version's notes by @debba
- *(query)* Execute SHOW and DESCRIBE without pagination wrapper (#5) by @vivekyarra in [#5](https://github.com/TabularisDB/tabularis-clickhouse-plugin/pull/5)
- *(manifest)* Add stable id and human-readable display name by @debba
## [0.2.0] - 2026-08-06

### Bug Fixes

- *(ci)* Upload the dotfile manifest artifact and attach it from artifacts by @debba
- *(manifest)* Lowercase plugin name to conform to the registry schema by @debba
- *(manifest)* Conform to the registry driver-kind schema by @debba

### Features

- Migrate to the .tabularium manifest for the Tabularium registry by @NewtTheWolf
- Add plugin icon to the .tabularium manifest by @debba
- Use a PNG icon in the manifest, keep the SVG source by @debba
## [0.1.1] - 2026-06-16

### Bug Fixes

- *(ssl)* Read ssl_mode from connection params instead of boolean ssl by @Aditeya

### Documentation

- Fix Discord shields.io badge server id by @debba

### Miscellaneous

- Switch Discord invite URL to discord.com/invite/K2hmhfHRSt by @debba
## [0.1.0] - 2026-03-07

### Bug Fixes

- *(query)* Avoid appending FORMAT JSONEachRow to SQL by @debba

### Documentation

- *(readme)* Add comprehensive usage and installation guide by @debba

### Features

- *(clickhouse)* Add initial plugin implementation and release pipeline by @debba
- *(api)* Add create/alter/drop view commands and update param names by @debba

### Miscellaneous

- Update cliff repo name and make sync.sh executable by @debba
- *(release)* Add tag-triggered multi-platform GitHub release workflow by @debba

