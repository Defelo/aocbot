use matrix_sdk::{
    ruma::events::room::member::{MembershipState, SyncRoomMemberEvent},
    Client, Room, RoomMemberships, RoomState,
};
use tracing::info;

pub async fn handle(event: SyncRoomMemberEvent, client: Client, room: Room) -> anyhow::Result<()> {
    if room.state() != RoomState::Joined || event.sender() == client.user_id().unwrap() {
        return Ok(());
    }

    if [MembershipState::Leave, MembershipState::Ban].contains(event.membership())
        && room
            .members(RoomMemberships::JOIN | RoomMemberships::INVITE)
            .await?
            .len()
            <= 1
    {
        info!("Leaving room {}", room.room_id());
        room.leave().await?;
    }

    Ok(())
}
