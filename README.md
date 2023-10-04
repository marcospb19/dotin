# dotin

A _"dotfiles manager"_ inspired by `stow`, and compatible with `stow`.

It uses symlinks to help you manage configuration files and versionate them with `git`.

`dotin` links files to your home directory (`~` or `$HOME`).

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
│   ├── batteryscript.sh
│   └── kb-layout-updater.sh
└── .config/
    └── polybar/
        ├── config.ini
        └── launch.sh
```

Let's call these files the "`polybar` **group**".

After hours of work, you probably want to backup these in order to:

1. Never lose the files.
2. Easily re-apply these changes in a new machine.
3. Keep new changes in sync between machines.

Tip: Check where the tools you use are saving their configuration files!

# How `dotin` helps

After you've done the setup, `dotin` lets you bulk-create symlinks from backup location to the
desired file location.

## The setup (hardest part)

First, create the folder that will hold all your configuration groups:

```sh
cd ~
mkdir dotfiles
cd dotfiles
```

Now, create the group folder `polybar`, and structure it like shown before:

```
dotfiles/
└── polybar/
    ├── .scripts/
    │   ├── volumescript.sh
    │   ├── batteryscript.sh
    │   └── kb-layout-updater.sh
    └── .config/
        └── polybar/
            ├── config.ini
            └── launch.sh
```

Files must be moved to the respective locations.

(Think of it this way: every relative path inside of the `polybar` group folder corresponds to the
same path in your _$HOME directory_.)

After that's done, run:

```sh
dotin link polybar
```

Files that live inside of the `dotfiles/polybar` folder are linked to their original home location,
but now they can live inside of a repository.

TODO: add importer helper.

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

`dotin`, right now, isn't much different from `stow`.

However, `dotin` is a newborn, and suscetible for change, I expect it to be, soon, incrementally
better, feature-wise.

## Current differences

- `dotin` creates intermediate folders when necessary, `stow` sometimes links them instead of the
files inside.

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
