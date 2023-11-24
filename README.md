**Note**: This project is _3 weeks old_, there are rough edges. You might want to wait a little more.

# dotin

An _Unix dotfiles manager_ inspired by `stow`.

It groups files in a way that allows you to manage your configs with `git`, this is good if you want to:

1. Backup your configs.
2. Easily re-apply them in another machine.
3. Keep new changes in sync between machines.

`dotin` and `stow` share the same file tree structure, so switching between both is effortless.

# Table Of Contents

- Problem statement
- How `dotin` helps
    - The setup (hardest part)
    - Sync with GitHub
    - Reapplying changes in a new machine
- Differences from `stow`
- Known limitations
- Alternatives

# Problem statement

As an example, imagine you just finished configuring a tool like [`polybar`].

Here's the overview, from your home directory, of what that might look like:

```ruby
├── .scripts/
│   ├── volumescript.sh
│   └── kb-layout.sh
└── .config/
    └── polybar/
        ├── config.ini
        └── launch.sh
```

Let's call these files the "**polybar group**".

After hours of work, you probably want to backup these files in order to (again):

1. Not lose the files.
2. Easily re-apply them in another machine.
3. Keep new changes in sync between machines.

# How `dotin` helps

With `dotin`, you group these files together, and for each file, create a symlink to the desired location.

## The setup (hardest part)

First, create the folder that will hold all your configuration groups:

```sh
cd ~
mkdir dotfiles
```

Now, create the group folder `polybar`, and structure it like shown before:

```ruby
dotfiles/
└── polybar/
    ├── .scripts/
    │   ├── volumescript.sh
    │   └── kb-layout.sh
    └── .config/
        └── polybar/
            ├── config.ini
            └── launch.sh
```

<details>
    <summary>What is this structure?</summary>

> Think of it this way: every path inside of the `polybar` group folder corresponds to the same path in your _home directory_.
>
> Examples:
>  - `~/file` -> `~/dotfiles/polybar/file`
>  - `~/path/to/file` -> `~/dotfiles/polybar/path/to/file`
>
> So you need to recreate that tree inside of the dotfiles folder.
</details>

You can move the files manually, or use the `dotin import` command:

```sh
dotin import polybar .config/polybar/* .scripts/{volumescript,kb-layout}.sh
```

This is the command syntax:

```sh
dotin import <GROUP_NAME> [FILES...]
```

<details>
    <summary>What is this syntax?</summary>

> This formatting (with `{a,b}` and `*`) is not a `dotin` feature.
>
> That's just a shell pattern expansion, works for any command in `bash`, `zsh`, and `fish`.
>
> Here are some references if you want to know more about it:
> - Zsh
>   - Brace expansion: https://zsh.sourceforge.io/Doc/Release/Expansion.html#Brace-Expansion
>   - Asterisk expansion (glob operator): https://zsh.sourceforge.io/Doc/Release/Expansion.html#Glob-Operators
> - Bash
>   - Brace expansion: https://www.gnu.org/software/bash/manual/html_node/Brace-Expansion.html
>   - Asterisk expansion (globstar) https://www.gnu.org/software/bash/manual/bash.html#Pattern-Matching
</details>

After that's done, run:

```sh
dotin link polybar
```

Now, each file is linked to its original location, but all files are grouped in a single folder.

## Sync with GitHub

```sh
cd ~/dotfiles
git init
# Go to GitHub, create a repo without README, follow the instructions that look like this:
git remote add origin URL
git commit -a
git push -u origin HEAD
```

Congrats, your configuration files are backed up in the cloud.

## Reapplying changes in a new machine

```sh
git clone URL dotfiles
cd dotfiles
dotin link polybar
```

Done, all configuration files are linked to the correct locations.

If there are any conflicts, they'll be reported, and you'll have to manually solve them.

Conflicts happen when there is a file at the desired link location, and creating a link would require erasing the file.

# Differences from `stow`

- The `import` subcommand.
- It creates intermediate directories when necessary.

That's not much, `dotin` is a newborn and might deviate more in the future.

Although `dotin` is a superset of `stow`, it aims to remain extremely simple.

# Known limitations

Things that I want to address:

- `dotin` can link regular files and directories, symlinks and other file types are not supported.
    - Symlinks were supposed to be supported, but Unix makes it almost impossible to canonicalize the location of a symlink (the symlink path, not the target path).
    - I think I have a workaround for that, but I didn't implement it yet.
- Changing dotfiles folder and home folder is not supported.

# Alternatives

- Use `stow` instead.
    - Its tree structure is the same, both tools are compatible with each other.
    - `stow` is shipped to most main distros package managers, that's a plus.
    - I'm using `dotin`, but when I need to link a group in a random system, I just install `stow`, clone my dotfiles, and link with it.
- Just create a script to link/copy files.
    - Valid, but if you're like me, you have [more than 10 groups to handle](https://github.com/marcospb19/dotfiles), a
    script for that is cumbersome.
- Make your `$HOME` directory a repository and `.gitignore` everything.
    - I don't like dealing with nested repositories.
    - Huge repos sometimes make my shell freeze (it got `git` integration).
    - If these don't bother you, you might like it, read [this](https://drewdevault.com/2019/12/30/dotfiles.html).
- Use [`dotbot`](https://github.com/anishathalye/dotbot) instead.
- Use [`mackup`](https://github.com/lra/mackup) instead.
- Use [`chezmoi`](https://github.com/twpayne/chezmoi) instead.
- Use [some other tool](https://wiki.archlinux.org/title/Dotfiles#Tools).
- Use [some `git`-wrapping tool](https://wiki.archlinux.org/title/Dotfiles#Tools_wrapping_Git).

[`polybar`]: https://github.com/polybar/polybar
