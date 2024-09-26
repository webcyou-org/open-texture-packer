# Open Texture Packer

Open source and free Texture Packer tool written in rust.

![twitter_header_photo_1](https://github.com/user-attachments/assets/75316555-b0e6-4465-b2f2-ac4943b25f39)

## Installation

### Home Brew

```
brew tap webcyou-org/tap
brew install open-texture-packer
```

### Cargo

```
cargo install open_texture_packer
```

The otp command is available upon installation.

```
otp <any option>
```

## Basic Usage

Specifies the path of the directory containing the images to be textured.

```
otp <input_directory>
```

You can also specify an arbitrary directory for the output destination

```
otp <input_directory> [output_directory]
```

If nothing is specified, images in the current directory are retrieved.

```
otp
```

In the development environment, it can also be run with cargo run.

```
cargo run <input_directory>
cargo run <input_directory> [output_directory]
cargo run
```

### Packing Algorithm

Currently, the process is similar to Shelf Packing and Next Fit. 

We plan to add them in the near future and make them selectable.

- Shelf Packing
- Next Fit
- Best Fit
- First Fit
- Worst Fit
- First Fit Decreasing
- Best Fit Decreasing
- Guillotine Cutting
- Skyline Algorithm
- Maximal Rectangles

## Author

**Daisuke Takayama**
* [@webcyou](https://twitter.com/webcyou)
* [@panicdragon](https://twitter.com/panicdragon)
* <https://github.com/webcyou>
* <https://github.com/webcyou-org>
* <https://github.com/panicdragon>
* <https://www.webcyou.com/>

### License

Copyright (c) 2024 Daisuke Takayama
Released under the [MIT license](http://opensource.org/licenses/mit-license.php)
