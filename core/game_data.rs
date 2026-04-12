use std::sync::LazyLock;

pub static DATA: LazyLock<data::DataSetInfo> =
    LazyLock::new(|| data::load("blastcap-data/Cap").unwrap());
