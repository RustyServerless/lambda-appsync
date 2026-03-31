macro_rules! weird_field_names {
    ($t:ident) => {
        let _weird = $t {
            r#as: true,
            r#async: true,
            r#await: true,
            r#break: true,
            r#const: true,
            r#continue: true,
            r_crate: true,
            r#dyn: true,
            r#else: true,
            r#enum: true,
            r#extern: true,
            r#false: true,
            r#fn: true,
            r#for: true,
            r#if: true,
            r#impl: true,
            r#in: true,
            r#let: true,
            r#loop: true,
            r#match: true,
            r#mod: true,
            r#move: true,
            r#mut: true,
            r#pub: true,
            r#ref: true,
            r#return: true,
            r_self: true,
            r#static: true,
            r#struct: true,
            r_super: true,
            r#trait: true,
            r#true: true,
            r#type: true,
            r#unsafe: true,
            r#use: true,
            r#where: true,
            r#while: true,
            r#abstract: true,
            r#become: true,
            r#box: true,
            r#do: true,
            r#final: true,
            r#macro: true,
            r#override: true,
            r#priv: true,
            r#try: true,
            r#typeof: true,
            r#unsized: true,
            r#virtual: true,
            r#yield: true,
            bool: true,
            char: "x".into(),
            f32: 1.0,
            f64: 1.0,
            i8: 1,
            i16: 1,
            i32: 1,
            i64: 1,
            i128: 1,
            isize: 1,
            str: "string".into(),
            u8: 1,
            u16: 1,
            u32: 1,
            u64: 1,
            u128: 1,
            usize: 1,
        };
    };
}

mod appsync_lambda_main {
    lambda_appsync::appsync_lambda_main!("../../../../schema.graphql", only_appsync_types = true);
    pub fn test() {
        weird_field_names!(WeirdFieldNames);
        let _optional = OptionalTeam {
            team: Some(Team::Rust),
        };
        let _optional_none = OptionalTeam { team: None };
    }
}
mod make_appsync {
    lambda_appsync::make_appsync!("../../../../schema.graphql");
    pub fn test() {
        weird_field_names!(WeirdFieldNames);
        let _optional = OptionalTeam {
            team: Some(Team::Rust),
        };
        let _optional_none = OptionalTeam { team: None };
    }
}
mod make_types {
    lambda_appsync::make_types!("../../../../schema.graphql");
    pub fn test() {
        weird_field_names!(WeirdFieldNames);
        let _optional = OptionalTeam {
            team: Some(Team::Rust),
        };
        let _optional_none = OptionalTeam { team: None };
    }
}

fn main() {
    appsync_lambda_main::test();
    make_appsync::test();
    make_types::test();
}
