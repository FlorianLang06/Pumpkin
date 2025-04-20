use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::block::entities::sign::SignBlockEntity;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

type SignProperties = pumpkin_data::block::OakSignLikeProperties;
type WallSignProps = pumpkin_data::block::LadderLikeProperties;

pub struct SignBlock;

impl BlockMetadata for SignBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:signs").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for SignBlock {
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        block: &Block,
        face: &BlockDirection,
        _block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player: &Player,
        _other: bool,
    ) -> u16 {
        if face.is_horizontal() {
            let wall_block = match get_wall_block(block) {
                Some(b) => b,
                None => {
                    log::error!("Failed to get the wall sign for {}", block.name);

                    Block::OAK_WALL_SIGN
                }
            };
            let mut props = WallSignProps::default(&wall_block);
            props.facing = match face.to_horizontal_facing() {
                Some(f) => f.opposite(),
                None => {
                    log::error!("Failed to get horizontal facing for sign");
                    return wall_block.default_state_id;
                }
            };
            return props.to_state_id(&wall_block);
        }

        let sign_props = SignProperties::default(block);

        sign_props.to_state_id(block)
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        _block: &Block,
        _state_id: u16,
        pos: &BlockPos,
        _old_state_id: u16,
        _notify: bool,
    ) {
        world
            .add_block_entity(Arc::new(SignBlockEntity::empty(*pos)))
            .await;
    }

    async fn player_placed(
        &self,
        _world: &Arc<World>,
        _block: &Block,
        _state_id: u16,
        pos: &BlockPos,
        _face: &BlockDirection,
        player: &Player,
    ) {
        player.send_sign_packet(*pos).await;
    }

    async fn on_state_replaced(
        &self,
        world: &Arc<World>,
        _block: &Block,
        location: BlockPos,
        _old_state_id: u16,
        _moved: bool,
    ) {
        world.remove_block_entity(&location).await;
    }
}

fn get_wall_block(block: &Block) -> Option<Block> {
    match block.name.chars().rev().position(|c| c == '_') {
        Some(rev_index) => {
            let index = block.name.chars().count() - rev_index - 1;
            let sign_type = &block.name[..index];
            let wall_block_name = format!("{}_wall_sign", &sign_type);
            Block::from_registry_key(wall_block_name.as_str())
        }
        None => None,
    }
}
