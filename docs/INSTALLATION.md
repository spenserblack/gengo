# Installation

## Install Script

Behind the scenes, this script downloads a binary from the latest GitHub release.

**This will activate `sudo` to write the executable.** Please review the install script before
running it.

### Unix

```shell
curl https://raw.githubusercontent.com/spenserblack/gengo/main/install.sh | sh
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
