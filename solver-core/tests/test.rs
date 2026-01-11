use solver_core::*;

#[test]
fn test_example() {
    let game = PostFlopGameInterface::new("2s3s4s".to_string(), 0);
    let _1 = game.action(1);
    let valid_actions_history_1 = game.get_valid_actions_history();
    assert_eq!(valid_actions_history_1.len(), 2);
    assert_eq!(valid_actions_history_1[0].opponent_bet_size, None);
    assert_eq!(valid_actions_history_1[1].opponent_bet_size, Some(5));
    let _2 = game.action(0);
    let valid_actions_history_2 = game.get_valid_actions_history();
    assert_eq!(valid_actions_history_2.len(), 3);
    assert_eq!(valid_actions_history_2[2].opponent_bet_size, None);
}
