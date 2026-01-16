# dot-manager

Simple dotfiles manager

## Usage

`dot-manager` can be used to synchronize your local files into a single directory.

Start by creating a configuration file at `~/.config/dot-manager/config.toml`.

The configuration requires the `remote_path` field. This is the path to the directory you want to
synchronize your files in.

Then you can provide paths related to a file using the `[[file]]` array. This field takes two
fields:
- `local`: the path to the file located in your filesystem. This field is required.
- `remote`: the path to the file located in the provided `remote_path` field. This field is
  optional, if not provided, the file will be written at the root of the directory.

You can avoid repeating `/home/user/` for the `remote_path` or `local` paths if the path doesn't
start with `/`. The full path is determined using the `$HOME` environment variable or you can
provide the path you want using the optional `home_path` field to override the environment variable.

Example:
```toml
remote_path = "dotfiles"
# remote_path = "/home/user/dotfiles" if you want to use a full path

home_path = "/home/user" # optional

[[file]]
local = ".gitignore"
# local = "/home/user/.gitignore" if you want to use a full path
remote = ".gitignore" # This will synchronize `.gitignore` to `dotfiles/.gitignore`

[[file]]
local = ".config/nvim/init.lua"
remote = "nvim/init.lua" # This will synchronize `.config/nvim/init.lua` to `dotfiles/nvim/init.lua`
```

Note that you need to provide a path to a file, you can't synchronize a full directory (yet?).

## Command-line

Launching `dot-manager` without argument gives you the current status of the synchronization between
the local and the remote files without changing any files.

The status output regroups files depending on the steps needed for synchronization:
- Up to date: Files that are synchronized between the local filesystem and the remote directory.
- To update: Files with content that is not synchronized between the local filesystem and the remote
  directory.
- To upload: Files available in the local filesystem but not present on the remote directory.
- To download: Files available in the remote directory but not present in the local filesystem.

Example:
```
Up to date:
* .bash_profile
* /boot/loader/loader.conf
* /boot/loader/entries/sanctuary.conf
* .config/i3status/config
* .config/nvim/init.lua
* .config/nvim/spell/en.utf-8.add
* .gitignore_global
* .xinitrc

To update:
* .bashrc - bash/.bashrc
* .config/starship.toml - bash/starship.toml
* .config/i3/config - i3/config
* .gitconfig - git/.gitconfig

To upload:
* .config/alacritty/alacritty.toml

To download:
* .Xresources
```

Additionally, there are arguments you can use that follow the output:
- `--update`: Update content from local or remote files. This field needs an additional argument:
    - `local`: Update local files from the remote directory.
    - `remote`: Update remote files from the local filesystem.
- `--upload`: Upload local files that don't exist yet in the remote directory.
- `--download`: Download remote files that don't exist in the local filesystem.

Note that you can use multiple arguments to do all the steps you want in one command.
