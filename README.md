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

### Rust

```
cargo add open_texture_packer
```

```
[dependencies]
open_texture_packer = "0.2.1"
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

### Result - Basic Usage

When performed when three sprites (sprite1.png, sprite2.png, sprite3.png) of different sizes are present in a directory

<img width="500" src="https://github.com/webcyou-org/open-texture-packer/blob/main/assets/screenshot/sprite_images.png">

The following files are generated in the output

```
texture_sheet_1.css
texture_sheet_1.json
texture_sheet_1.png
```

The texture_sheet_1.png is merged to produce a texture image as follows

<img width="500" src="https://github.com/webcyou-org/open-texture-packer/blob/main/assets/screenshot/texture_image.png">


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
