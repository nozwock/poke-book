# PokeBook

Your ultimate desktop companion for Pokémon information and fun!
> <small>Made for the [MLH GHW 2024: API](https://ghw.mlh.io/) event.</small>

![](https://github.com/nozwock/poke-book/assets/57829219/6a10cdcc-6095-4365-b2f9-f224c8382b1b)

## Building the project
Make sure you have `flatpak` and `flatpak-builder` installed. Then run the commands below.
```shell
meson setup build

flatpak install --user org.gnome.Sdk//46 org.gnome.Platform//46  org.freedesktop.Sdk.Extension.rust-stable//23.08 org.freedesktop.Sdk.Extension.llvm16//23.08

flatpak-builder --user _build build-aux/com.github.nozwock.PokeBook.Devel.json
```

## Running the project
Once the project is build, run the command below.
```shell
flatpak-builder --run _build build-aux/com.github.nozwock.PokeBook.Devel.json poke-book
```
