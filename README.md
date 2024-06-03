# fast-to-sportsmanager
Command-line utility program to convert FAST .xml tournament results into a number of .xml files that can be imported in sportsmanager

# Functionality
- parses FAST .xml output file
- scrapes player names from tablesoccer.org (the FAST output file only contains ITSF license IDs)
- maintains ITSF player name cache (`player_cache.json`)
- groups FAST competitions so that qualifications and A/B-eliminations go together
- generates a set of files for each distinct competition

# Build
- get Rust (see https://rustup.rs/)
- `cargo run [path to FAST .xml file]`