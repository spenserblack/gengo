# Installation

## Install Script

Behind the scenes, this script downloads a binary from the latest GitHub release.

### Windows

```powershell
Invoke-RestMethod "https://raw.githubusercontent.com/spenserblack/gengo/main/install.ps1" | Invoke-Expression
```

If you get an error regarding execution policy, please read the error carefully and determine the execution policy that is right for you. You may try re-running the installation script if you have updated the execution policy.

### Unix

**This will activate `sudo` to write the executable.** Please review the install script before
running it.

```shell
curl https://raw.githubusercontent.com/spenserblack/gengo/main/install.sh | sh
```

## Arch Linux

gengo is available as [an AUR package](https://aur.archlinux.org/packages/gengo/).
It can be installed using an AUR helper (e.g. paru):

```shell
paru -S gengo
```

## Scoop

gengo is available in [Scoop](https://scoop.sh). Note that manifest [published only as GitHub Gist](https://gist.github.com/ankddev/0f5ba567eb70b52825aa92b3a3532494)
```shell
scoop install https://gist.githubusercontent.com/ankddev/0f5ba567eb70b52825aa92b3a3532494/raw/gengo.json
```

## From GitHub Releases

Download the appropriate file from [a release](https://github.com/spenserblack/gengo/releases).

## With `cargo`

The following installs the `gengo` binary.

```shell
cargo install gengo-bin
```

## Docker

You can build a docker image and run it as well.

```bash
docker build -t gengo .
docker run --rm -v $(pwd):$(pwd) -w $(pwd) -t gengo
```

Or pull it from the [GitHub Container Registry][ghcr-package].

[ghcr-package]: https://github.com/users/spenserblack/packages/container/package/gengo
