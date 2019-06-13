### Rust Vulkan

an attempt at writing a general refraction shader in vulkan with rust as an excercise and learning experience. most code is boiler plate, but can be run wafter following some prereqs.

install the vulkan libraries as described in (this)[https://github.com/vulkano-rs/vulkano/blob/master/README.md] readme. 

then run the src with cargo on any wavefront object file

```bash
cargo run -- --input test_objs/teapot.obj
```

it may exit with a panic if your file is in the wrong format. or if you resize the window too fast.

---

currently only supports single objects

---

TODO : 

[] skybox
[] refraction in glsl
[] refactor code into structs
