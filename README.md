# About
This editor edits s-expressions structurally, and is designed around that idea. It is a modal editor with few keybinds and is easy to learn.

# Keybinds in Structural mode (S)
- `"` or `'` creates a string
- `l` creates a list
- `i` creates an identifier
- `n` creates a number
- `Tab` moves to the next object
- `Shift+Tab` moves to the previous object
- `Enter` moves into the object under the cursor
- `Esc` moves out of the current object
- `Delete` deletes the object under the cursor
- `:` enters command mode

# Keybinds in Edit mode (E)
- `Esc` exits edit mode
- `Left arrow` and `Right arrow` moves the cursor around
- `Backspace` deletes the character to the left of the cursor and moves left one
- `Delete` deletes the character under the cursor
- Any printable key is inserted into the current object

# Keybinds in Command mode (C)
- `Esc` clears the command and goes back to Structural mode
- `Enter` executes the command

# Commands
Currently, there are only a couple variations of 3 commands:
- Quit `:q`
- Write file `:w <?FILENAME>`
- Quit with force `:q!`

## Variations on the commands
The commands `:wq <?FILENAME>`, `:wq! <?FILENAME>`, are also valid and work as expected
