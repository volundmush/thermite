table! {
    accountcomponent (id) {
        id -> Uuid,
        password -> Text,
        email -> Varchar,
        superuser -> Bool,
    }
}

table! {
    aclentry (id) {
        id -> Int8,
        resource -> Uuid,
        target -> Uuid,
        mode -> Varchar,
        deny -> Bool,
    }
}

table! {
    acllink (id) {
        id -> Int8,
        acl_id -> Int8,
        perm_id -> Int4,
    }
}

table! {
    aclpermission (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    attributes (id) {
        id -> Int8,
        owner -> Uuid,
        category -> Varchar,
        name -> Varchar,
        data -> Json,
    }
}

table! {
    childcomponent (id) {
        id -> Uuid,
        parent -> Uuid,
        child_type_id -> Int4,
        child_key -> Varchar,
    }
}

table! {
    childtype (id) {
        id -> Int4,
        name -> Nullable<Varchar>,
    }
}

table! {
    entities (id) {
        id -> Uuid,
        type_id -> Int4,
        python_path -> Varchar,
    }
}

table! {
    entitylocationcomponent (id) {
        id -> Uuid,
        location -> Uuid,
    }
}

table! {
    equipslotcomponent (id) {
        id -> Uuid,
        slot_key -> Varchar,
        slot_layer -> Int4,
    }
}

table! {
    fixturecomponent (id) {
        id -> Uuid,
        fixture_space_id -> Int4,
        fixture_key -> Varchar,
    }
}

table! {
    fixturespace (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    floatpositioncomponent (id) {
        id -> Uuid,
        x -> Float8,
        y -> Float8,
        z -> Float8,
    }
}

table! {
    intpositioncomponent (id) {
        id -> Uuid,
        x -> Int4,
        y -> Int4,
        z -> Int4,
    }
}

table! {
    namecomponent (id) {
        id -> Uuid,
        namespace_id -> Int4,
        name -> Varchar,
        color_name -> Varchar,
    }
}

table! {
    namespace (id) {
        id -> Int4,
        name -> Varchar,
        searchable -> Bool,
    }
}

table! {
    playercharactercomponent (id) {
        id -> Uuid,
        account_id -> Uuid,
    }
}

table! {
    pluginname (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    types (id) {
        id -> Int4,
        name -> Varchar,
    }
}

joinable!(accountcomponent -> entities (id));
joinable!(acllink -> aclentry (acl_id));
joinable!(acllink -> aclpermission (perm_id));
joinable!(attributes -> entities (owner));
joinable!(childcomponent -> childtype (child_type_id));
joinable!(entities -> types (type_id));
joinable!(equipslotcomponent -> entities (id));
joinable!(fixturecomponent -> entities (id));
joinable!(fixturecomponent -> fixturespace (fixture_space_id));
joinable!(floatpositioncomponent -> entities (id));
joinable!(intpositioncomponent -> entities (id));
joinable!(namecomponent -> entities (id));
joinable!(namecomponent -> namespace (namespace_id));

allow_tables_to_appear_in_same_query!(
    accountcomponent,
    aclentry,
    acllink,
    aclpermission,
    attributes,
    childcomponent,
    childtype,
    entities,
    entitylocationcomponent,
    equipslotcomponent,
    fixturecomponent,
    fixturespace,
    floatpositioncomponent,
    intpositioncomponent,
    namecomponent,
    namespace,
    playercharactercomponent,
    pluginname,
    types,
);
