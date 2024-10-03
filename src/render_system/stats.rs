use crate::objects::get_armor_class;
use crate::player::Player;
use crate::resources::avatar::Avatar;

pub(crate) fn format_stats_legacy(player: &Player) -> String {
	let level = format!("Level: {:<2}", player.cur_depth);
	let gold = format!("Gold: {:<6}", player.gold());
	let hit_points = format!("Hp: {:<8}", format!("{}({})", player.rogue.hp_current, player.rogue.hp_max));
	let strength = format!("Str: {:<6}", format!("{}({})", player.buffed_strength(), player.max_strength()));
	let armor = format!("Arm: {:<2}", get_armor_class(player.armor()));
	let experience = format!("Exp: {:<11}", format!("{}/{}", player.rogue.exp.level, player.rogue.exp.points));
	let hunger = player.rogue_energy().as_stat();
	format!("{level} {gold} {hit_points} {strength} {armor} {experience} {hunger}")
}
pub fn format_stats(avatar: &impl Avatar) -> String {
	let depth = avatar.rogue_depth();
	let fighter = avatar.as_fighter();

	let level = format!("Level: {:<2}", depth);
	let gold = format!("Gold: {:<6}", fighter.gold);
	let hit_points = format!("Hp: {:<8}", format!("{}({})", fighter.hp_current, fighter.hp_max));
	let strength = format!("Str: {:<6}", format!("{}({})", avatar.buffed_strength(), avatar.max_strength()));
	let armor = format!("Arm: {:<2}", get_armor_class(avatar.armor()));
	let experience = format!("Exp: {:<11}", format!("{}/{}", fighter.exp.level, fighter.exp.points));
	let hunger = avatar.rogue_energy().as_stat();
	format!("{level} {gold} {hit_points} {strength} {armor} {experience} {hunger}")
}
