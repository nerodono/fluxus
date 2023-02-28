use appconf::macros::decl_config;

#[decl_config(loader = "toml")]
pub struct Test {
    pub nero: String,
}

fn main() {
    let a = Test {
        nero: "nero".to_owned(),
    };

    let serialized = a.serialize(true);
    let deserialized = Test::try_parse(&serialized).unwrap();

    println!("{serialized}");
    dbg!(deserialized);
}
