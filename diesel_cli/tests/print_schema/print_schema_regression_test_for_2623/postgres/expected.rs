// @generated automatically by Diesel CLI.

diesel::table! {
    tab1 (id) {
        id -> Int8,
    }
}

diesel::table! {
    tab_problem (id) {
        id -> Int8,
        key1 -> Int8,
    }
}

diesel::joinable!(tab_problem -> tab1 (key1));

diesel::allow_tables_to_appear_in_same_query!(
    tab1,
    tab_problem,
);
