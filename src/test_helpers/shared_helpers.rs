use std::ops::Bound;

use chrono::{Duration, SubsecRound, Utc};
use diesel::pg::PgConnection;
use rand::Rng;
use svc_agent::AgentId;
use uuid::Uuid;

use crate::db::agent::{Object as Agent, Status as AgentStatus};
use crate::db::janus_backend::Object as JanusBackend;
use crate::db::recording::Object as Recording;
use crate::db::room::{Object as Room, RoomBackend};
use crate::db::rtc::Object as Rtc;

use super::{agent::TestAgent, factory, SVC_AUDIENCE, USR_AUDIENCE};

///////////////////////////////////////////////////////////////////////////////

pub(crate) fn insert_room(conn: &PgConnection) -> Room {
    let now = Utc::now().trunc_subsecs(0);

    factory::Room::new()
        .audience(USR_AUDIENCE)
        .time((Bound::Included(now), Bound::Unbounded))
        .backend(RoomBackend::Janus)
        .insert(conn)
}

pub(crate) fn insert_closed_room(conn: &PgConnection) -> Room {
    let now = Utc::now().trunc_subsecs(0);

    factory::Room::new()
        .audience(USR_AUDIENCE)
        .time((
            Bound::Included(now - Duration::hours(10)),
            Bound::Excluded(now - Duration::hours(8)),
        ))
        .backend(RoomBackend::Janus)
        .insert(conn)
}

pub(crate) fn insert_agent(conn: &PgConnection, agent_id: &AgentId, room_id: Uuid) -> Agent {
    factory::Agent::new()
        .agent_id(agent_id)
        .room_id(room_id)
        .status(AgentStatus::Connected)
        .insert(conn)
}

pub(crate) fn insert_janus_backend(conn: &PgConnection) -> JanusBackend {
    let mut rng = rand::thread_rng();

    let label_suffix: String = rng
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(5)
        .collect();
    let label = format!("janus-gateway-{}", label_suffix);

    let agent = TestAgent::new("alpha", &label, SVC_AUDIENCE);
    factory::JanusBackend::new(agent.agent_id().to_owned(), rng.gen(), rng.gen()).insert(conn)
}

pub(crate) fn insert_rtc(conn: &PgConnection) -> Rtc {
    let room = insert_room(conn);
    factory::Rtc::new(room.id()).insert(conn)
}

pub(crate) fn insert_rtc_with_room(conn: &PgConnection, room: &Room) -> Rtc {
    factory::Rtc::new(room.id()).insert(conn)
}

pub(crate) fn insert_recording(
    conn: &PgConnection,
    rtc: &Rtc,
    backend: &JanusBackend,
) -> Recording {
    factory::Recording::new()
        .rtc(rtc)
        .backend(backend)
        .insert(conn)
}
