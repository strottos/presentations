# Vim - The Good, The Bad, and The Downright Beautiful

## Outline of Talk

- Who am I, why am I qualified to give this talk and what is this talk about?
<!-- pause -->
- What is Vim/Neovim and why are they great?
<!-- pause -->
- What are the differences between Vim & Neovim?

<!-- end_slide -->

# Who am I, why am I qualified to give this talk and what is this talk about?

- 10+ years experience in software development and 5 years in DevOps
<!-- pause -->
- Never had more than maybe a continuous maximum of a week of using anything but Vim/Neovim as my primary editor
<!-- pause -->
- I have spent time however learning about how IDEs work, how they are built and what's cool about them
<!-- pause -->
- I've written a few plugins for Vim/Neovim and have a good understanding of how they work
<!-- pause -->
- I've spent approximately forever perfecting (not even close) my Vim/Neovim setup

<!-- end_slide -->

# What is Vim/Neovim and why are they great?

- Vim is a modal text editor that is designed to be fast and efficient
<!-- pause -->
- Vim is super super powerful as a text editor
<!-- pause -->
- Vim is highly configurable and extensible, I can change:
  - keybindings
  - the look and feel
  - the functionality
  - I can even write my own/use existing plugins to make it feel more like an IDE
<!-- pause -->
- Vim is super fast, I can open a file in Vim in less than a second and start editing it immediately
<!-- pause -->
- Super super important for me, I can use Vim basically anywhere, I can use it on my laptop, on a server, on a Raspberry
  Pi, on a Windows machine, on a Mac, my Phone, on a random Container (mostly anyway, will come back to this one), you
  name it, I can use Vim on it.
<!-- pause -->
- All this is true of Neovim as well
<!-- end_slide -->

# What are the differences between Vim & Neovim?

- Neovim is a fork of Vim that aims to modernise the codebase and make it easier to extend
<!-- pause -->
- Neovim has a lot of features that Vim doesn't have, not necessarily out of the box, but it can be configured
reasonably easy to do, like:
  - LSP support
  - Treesitter support
  - debugging support
  - Git integration
  - Agentic Coding support
(more on these later)
<!-- pause -->
- Neovim's killer feature is that it can be configured with Lua, which is a much more powerful and flexible language
  than Vimscript in Vim. This means that Neovim can be extended in ways that is much harder to do in Vim (but not impossible).
<!-- pause -->
- Neovim is more actively developed than Vim, particularly true of the plugin world.
<!-- pause -->
BUT...
<!-- pause -->
- Stability of Vim is unprecedented, stability of Neovim not so much.
<!-- pause -->
- Also, Lua, I mean really, Lua?
<!-- end_slide -->

# How can I turn them into an IDE?

- Vim, as mentioned, is a text editor, not an IDE, but it can be configured to feel like one. But be prepared to spend a
  lot of time configuring it and not being able to find plugins that do what you want.
<!-- pause -->
- Neovim, pretty much the same, but it has a lot more options for configuration and a lot more plugins that can help
  you achieve this.
<!-- pause -->
* Let's take a look over my Neovim setup and how I have configured it to feel like an IDE.
<!-- end_slide -->

# Should I use Vim, Neovim or an IDE?

<!-- pause -->
IT DEPENDS!!!
<!-- pause -->
To the floor...
