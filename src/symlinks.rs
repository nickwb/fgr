// #[macro_use]
// extern crate clap;

arg_enum! {
    #[derive(PartialEq, Debug)]
    pub enum SymlinkBehaviour {
        Skip,
        Follow
    }
}
