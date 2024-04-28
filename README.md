# gui

Gui is a novel way to create graphical user interfaces in Rust.

To get started create a layout file in YAML to define the appearance of your application.

```yaml
components:
  - name: HelloWorld
    variables:
      - name: name
        type: String
    child:
      widget: Text
      properties:
        text: "Hello { $name }!"
        size: 30
```

And then create a user defined component that matches the name defined in the layout file.

```rust
#[derive(ToComponent)]
struct HelloWorld {
    name: Updateable<String>,
}

type_registry!();

fn main() {
    gui::run(HelloWorld {
        name: Updateable::new(String::from("world"))
    });
}
```

This will create a window with the text "Hello world!".

![image](https://github.com/miam-miam/gui/assets/49870539/bd21bdb9-1459-4157-89a4-1d08143c86d8)

## Dependencies

Gui requires a recent rust toolchain to build; it does not (yet) have an
explicit minimum supported rust version, but the latest stable version should
work.

On Linux and BSD, Gui also requires `pkg-config` and `clang`,
and the development packages of `wayland`, `libxkbcommon` and `libxcb`, to be installed.
Some of the examples require `vulkan-loader`.

Most distributions have `pkg-config` installed by default. To install the remaining packages on Fedora, run

```sh
sudo dnf install clang wayland-devel libxkbcommon-x11-devel libxcb-devel vulkan-loader-devel
```

To install them on Debian or Ubuntu, run

```sh
sudo apt-get install pkg-config clang libwayland-dev libxkbcommon-x11-dev libvulkan-dev
```

## Examples

Once you have the dependencies installed, you can run the examples found in [examples](examples) with:

```sh
cargo run --package <example-name>
```

## Why is it called gui?

To prevent bike shedding, I'll come up with a better name when the project needs one.

### CI

To run tests in CI install a software render

```yaml
- name: install llvmpipe and lavapipe (sofware based renderers)
  shell: bash
  run: |
    set -e

    sudo apt-get update -y -qq

    # vulkan sdk
    wget -qO - https://packages.lunarg.com/lunarg-signing-key-pub.asc | sudo apt-key add -
    sudo wget -qO /etc/apt/sources.list.d/lunarg-vulkan-jammy.list https://packages.lunarg.com/vulkan/lunarg-vulkan-jammy.list

    sudo add-apt-repository ppa:kisak/kisak-mesa

    sudo apt-get update
    sudo apt install -y libegl1-mesa libgl1-mesa-dri libxcb-xfixes0-dev vulkan-sdk mesa-vulkan-drivers
```

## Styling

Default styles are currently adapted from https://github.com/mantinedev/mantine.
