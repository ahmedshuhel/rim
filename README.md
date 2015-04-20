# Rim
Rim is an aspiring Vim-like text editor written in Rust.

## Current state
Focus has been put on building solid foundations free of code tangling rather than reaching quick goals. Thus many buffer operations required for rim to be called a text editor are lacking but will be easily implemented at the appropriate time. For now you can navigate multiple buffers in multiple split views and perform simple insertion.

## Try it out

Simply clone and `cargo run`.

Note: All key bindings are for testing purposes only.
- `q` - Quit
- `h/j/k/l/Arrow keys` - Move caret
- `<C-w>v/<C-w>s` - Split focused window
- `<C-w>c` - Close focused window
- `<C-w> h/j/k/l` - Move window focus left/down/up/right
- `n/N` - Move window focus forward/backward in window order
- `(<C->) y/u` - Resize focused window
- `<C-w>=` - Reset window sizes
- `i` - Enter insert mode
- `Escape` - Exit insert mode
- `F1-F4` - Load some buffers

## What's next?
A few more buffer operations to make rim feel a bit more homey and meet the basic requirements to be called a text editor. Then we'll see.

## Trouble shooting
Rustc updates are still a bit of a pain point. If things are not compiling, you're probably using a different version than I. I try to update at least once a week.