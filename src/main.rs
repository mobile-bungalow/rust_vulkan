/// Imports for loading files
use std::path::Path;
use tobj;

/// arg parse
use clap::{Arg,App};

mod vk_init;

fn main() {

// arg parsing, fails the program without input file
let matches = App::new("vk_obj")
                        .version("1.0")
                        .author("Paul May, pwmay@ucsc.edu")
                        .about("cross platform Obj loader in rust using vulkan bindings")
                        .arg(Arg::with_name("input")
                            .required(true)
                            .short("i")
                            .long("input")
                            .value_name("fname")
                            .help("the name of the input wavefront .obj file")
                            .takes_value(true))
                        .get_matches();

    // parse object with tiny object loader
    let obj_file = tobj::load_obj(&Path::new(matches.value_of("input").unwrap()));
    assert!(obj_file.is_ok());
    let (models, materials) = obj_file.unwrap();
    

    // init vulkan instance
     let mut vks : vk_init::VKState = vk_init::vk_setup();
        

}

#[cfg(test)]
mod tests {
    /// Tests objloader
    #[test]
    fn parse_obj() {}
}
