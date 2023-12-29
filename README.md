# TUILog - Because GUIs Crash on my Macbook

I wanted a very simple and functional logging software for all platforms. So, I wrote a TUI in Rust. No worrying about compiling graphics libraries!

Uses SQLite as persistent storage on the user (~/.tuilog/tuilog.db), and will export to ADIF

## Available Fields

### Operator

- Callsign
- Grid Square
- CQZ
- ITUZ
- DXCC
- Continent

## Log

- Callsign
- Band
- Frequency
- Mode
  - SSB
    - USB
    - LSB
  - CW
  - FT8
- RST TX/RX
- Power
- Comments

Make an Issue or PR to see more features!
