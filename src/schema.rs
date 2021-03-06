table! {
    use diesel::sql_types::*;
    use crate::db::sql::*;

    agent (id) {
        id -> Uuid,
        agent_id -> Agent_id,
        room_id -> Uuid,
        created_at -> Timestamptz,
        status -> Agent_status,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::sql::*;

    agent_stream (id) {
        id -> Uuid,
        sent_by -> Uuid,
        label -> Text,
        created_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::sql::*;

    janus_backend (id) {
        id -> Agent_id,
        handle_id -> Int8,
        session_id -> Int8,
        created_at -> Timestamptz,
        capacity -> Nullable<Int4>,
        balancer_capacity -> Nullable<Int4>,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::sql::*;

    janus_rtc_stream (id) {
        id -> Uuid,
        handle_id -> Int8,
        rtc_id -> Uuid,
        backend_id -> Agent_id,
        label -> Text,
        sent_by -> Agent_id,
        time -> Nullable<Tstzrange>,
        created_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::sql::*;

    recording (rtc_id) {
        rtc_id -> Uuid,
        started_at -> Nullable<Timestamptz>,
        segments -> Nullable<Array<Int8range>>,
        status -> Recording_status,
        backend_id -> Agent_id,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::sql::*;

    room (id) {
        id -> Uuid,
        time -> Tstzrange,
        audience -> Text,
        created_at -> Timestamptz,
        backend -> Room_backend,
        reserve -> Nullable<Int4>,
        tags -> Json,
    }
}

table! {
    use diesel::sql_types::*;
    use crate::db::sql::*;

    rtc (id) {
        id -> Uuid,
        room_id -> Uuid,
        created_at -> Timestamptz,
    }
}

joinable!(agent -> room (room_id));
joinable!(agent_stream -> agent (sent_by));
joinable!(janus_rtc_stream -> janus_backend (backend_id));
joinable!(janus_rtc_stream -> rtc (rtc_id));
joinable!(recording -> rtc (rtc_id));
joinable!(rtc -> room (room_id));

allow_tables_to_appear_in_same_query!(
    agent,
    agent_stream,
    janus_backend,
    janus_rtc_stream,
    recording,
    room,
    rtc,
);
