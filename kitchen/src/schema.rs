table! {
    categories (id) {
        id -> Nullable<Text>,
        categories -> Nullable<Text>,
    }
}

table! {
    recipes (id) {
        id -> Nullable<Text>,
        recipe -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    categories,
    recipes,
);
