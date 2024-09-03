pub mod gprocess {
    pub mod api {
        include!(concat!(env!("OUT_DIR"), "/gprocess.api.rs"));
    }
}
