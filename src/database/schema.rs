// @generated automatically by Diesel CLI.

diesel::table! {
    posts (id) {
        id -> Text,
        author -> Text,
        score -> Integer,
        title -> Text,
        created -> Float,
        body -> Nullable<Text>,
        link -> Nullable<Text>,
        category -> Nullable<crate::types::CategoryMapping>,
    }
}
