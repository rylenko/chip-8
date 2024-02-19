<h1 align="center">Welcome to Chip-8 🌿</h1>
Implementation of the Chip-8 emulator.

<h1 align="center">Installation</h1>

**1.** Clone this repository how you like it.

**2.** Check out the list of ROMs:
```
$ ls roms
```

**3.** Run the ROM:
```
$ cargo run <filename>
```

<h1 align="center">Keyboard</h1>

Chip-8 keyboard:
| | | | |
|-|-|-|-|
|1|2|3|C|
|4|5|6|D|
|7|8|9|E|
|A|0|B|F|

Is mapped to:
| | | | |
|-|-|-|-|
|1|2|3|4|
|Q|W|E|R|
|A|S|D|F|
|Z|X|C|V|

<h1 align="center">Controls in demo ROMs (games)</h1>

### Tetris:
* Q - Rotate piece
* W - Move left
* E - Move right
* A - Fast drop

### Invaders:
* Q - Move left
* W - Shoot weapon
* E - Move right

### Pong:
* 1 - Move the left racket up
* Q - Move the left racket down
* 4 - Move the right racket up
* R - Move the right racket down
