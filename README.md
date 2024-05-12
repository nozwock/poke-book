# PokeBook

Your ultimate desktop companion for Pokémon information and fun!
> <small>Made for the [MLH GHW 2024: API](https://ghw.mlh.io/) event.</small>

![](https://github.com/nozwock/poke-book/assets/57829219/6a10cdcc-6095-4365-b2f9-f224c8382b1b)

## Building the Project

Before building the project, ensure that you have the following dependencies installed: `meson`, `flatpak`, and `flatpak-builder`.

### Setup Configuration

First, set up the project configuration using Meson:

```shell
meson setup build
```

### Build and Run Options

You have two options for building and running the project:

1. **Install and Run via Meson**:
    ```shell
    meson -C build install
    poke-book
    ```

2. **Build and Run via Flatpak**:
    ```shell
    flatpak install --user \
        org.gnome.Sdk//46 \
        org.gnome.Platform//46 \
        org.freedesktop.Sdk.Extension.rust-stable//23.08 \
        org.freedesktop.Sdk.Extension.llvm16//23.08

    flatpak-builder --user build \
        build-aux/com.github.nozwock.PokeBook.Devel.json

    flatpak-builder --run build \
        build-aux/com.github.nozwock.PokeBook.Devel.json \
        poke-book
    ```

Choose the method that best suits your workflow and environment.
