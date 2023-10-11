**Note**: This project is 3 weeks old, there are rough edges. You might want to wait a little more.

# dotin

An _Unix dotfiles manager_ inspired by, and a superset of, `stow`.

It creates symlinks to allow you to versionate your configuration files with `git`.

`dotin` is compatible with `stow`, both tools share the same file tree structure.

# Table Of Contents

- Problem statement.
- How `dotin` helps.
    - The setup (hardest part).
    - Sync with GitHub.
    - Reapplying changes in a new machine.
- Alternatives.

# Problem statement

As an example, imagine you just finished configuring a tool like [`polybar`].

Here's the overview, from your home directory, of what it might look like:

```
~/
├── .scripts/
│   ├── volumescript.sh
│   └── kb-layout.sh
└── .config/
    └── polybar/
        ├── config.ini
        └── launch.sh
```

Let's call these files the "`polybar` **group**".

After hours of work, you probably want to backup these files in order to:

1. Not lose the files.
2. Easily re-apply them in another machine.
3. Keep new changes in sync between machines.

# How `dotin` helps

After you've done the setup, `dotin` lets you bulk-create symlinks from backup location to the
desired file location.

## The setup (hardest part)

First, create the folder that will hold all your configuration groups:

```sh
cd ~
mkdir dotfiles
```

Now, create the group folder `polybar`, and structure it like shown before:

```
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

(Think of it this way: every path inside of the `polybar` group folder corresponds to the
same path in your _home directory_.)

Files must be moved to the respective locations.

You move manually or use the `dotin import` command:

```
dotin import .config/polybar/* .scripts/{volumescript,kb-layout}.sh
```

<!-- TODO: check if this is called "expansion syntax" -->
Note: if you're not familiar, this formatting is not a `dotin` feature, but the expansion syntax for your shell (`bash`, `zsh`, and `fish`).

After that's done, run:

```sh
dotin link polybar
```

Files that live inside of the `polybar` group folder are linked to their original home location,
but now they actually live inside of a repository.

## Sync with GitHub

Go to GitHub and create a repo.

```sh
cd ~/dotfiles
git init
# Just do the usual setup
git remote add origin URL
git commit -a
git push -u origin HEAD
```

Congrats, your configuration files are backed-up in the cloud.

## Reapplying changes in a new machine

```sh
git clone URL dotfiles
cd dotfiles
dotin link polybar
```

Done, all configuration files were linked to their desired locations.

# Differences from `stow`

- `dotin` emits a more helpful output.
- The `import` subcommand.
- It creates intermediate directories when necessary.

`dotin` is an extremely new project, I have stuff in mind to expand it.

# Known limitations

- `dotin` can link regular files and directories, symlinks and other file types are not supported.
    - Symlinks were supposed to be supported, but Unix makes it impossible to canonicalize the location of a symlink.
    - I think I have a workaround for that, but I didn't implemented that, yet.
- Changing dotfiles folder and home folder is not supported yet.

# Alternatives

- Use `stow` instead.
    - Its folder structure is identical to `dotin`'s, both tools are compatible with each other.
    - `stow` is shipped to most main distros package managers, that's a plus.
- Just create a script to link/copy files.
    - Valid, but if you're like me, you have [more than 10 groups to handle](https://github.com/marcospb19/dotfiles), a
    script for that is cumbersome.
- Make your `$HOME` directory a repository and `.gitignore` everything.
    - I don't like dealing with nested repositories.
    - Huge repos sometimes make my shell freeze (it got `git` integration).
    - If these don't bother you, read [this](https://drewdevault.com/2019/12/30/dotfiles.html).
- Use [`dotbot`](https://github.com/TheLocehiliosan/dotbot) instead.
- Use [`mackup`](https://github.com/lra/mackup) instead.
- Use [`chezmoi`](https://github.com/twpayne/chezmoi) instead.
- Use [some other tool](https://wiki.archlinux.org/title/Dotfiles#Tools).
- Use [some `git`-wrapping tool](https://wiki.archlinux.org/title/Dotfiles#Tools_wrapping_Git).

[`polybar`]: https://github.com/polybar/polybar
